//! The OS keyring that holds the vault's data key. gpui's credentials
//! calls reach it, except inside a Flatpak: there its keyring library
//! keeps a separate per-app file instead, which wouldn't have the key a
//! native install stored for the same vault (and Crow, finding none, would
//! make a new one and drop the secrets it can't read). In a Flatpak Crow
//! talks to the session's Secret Service itself, as the same item gpui
//! writes, so both installs share one key.

use gpui_kit::*;

/// gpui's label for its credentials items; matched so either side finds
/// what the other wrote.
const LABEL: &str = "zed-github-account";

pub fn read(cx: &App, url: &str) -> Task<Result<Option<(String, Vec<u8>)>, String>> {
    if !crate::sandbox::flatpak() {
        let read = cx.read_credentials(url);
        return cx.background_executor().spawn(async move { read.await.map_err(|e| format!("{e:#}")) });
    }
    let url = url.to_string();
    cx.background_executor().spawn(async move {
        let keyring = session_keyring().await?;
        for item in keyring.search_items(&vec![("url", url.as_str())]).await.map_err(text)? {
            if item.label().await.is_ok_and(|l| l == LABEL) {
                let attributes = item.attributes().await.map_err(text)?;
                let user = attributes.get("username").cloned().ok_or("Cannot find username in stored credentials")?;
                item.unlock().await.map_err(text)?;
                return Ok(Some((user, item.secret().await.map_err(text)?.to_vec())));
            }
        }
        Ok(None)
    })
}

pub fn write(cx: &App, url: &str, user: &str, secret: &[u8]) -> Task<Result<(), String>> {
    if !crate::sandbox::flatpak() {
        let write = cx.write_credentials(url, user, secret);
        return cx.background_executor().spawn(async move { write.await.map_err(|e| format!("{e:#}")) });
    }
    let (url, user, secret) = (url.to_string(), user.to_string(), secret.to_vec());
    cx.background_executor().spawn(async move {
        let keyring = session_keyring().await?;
        keyring.create_item(LABEL, &vec![("url", url.as_str()), ("username", user.as_str())], secret, true).await.map_err(text)
    })
}

pub fn delete(cx: &App, url: &str) -> Task<Result<(), String>> {
    if !crate::sandbox::flatpak() {
        let delete = cx.delete_credentials(url);
        return cx.background_executor().spawn(async move { delete.await.map_err(|e| format!("{e:#}")) });
    }
    let url = url.to_string();
    cx.background_executor().spawn(async move {
        let keyring = session_keyring().await?;
        for item in keyring.search_items(&vec![("url", url.as_str())]).await.map_err(text)? {
            if item.label().await.is_ok_and(|l| l == LABEL) {
                item.delete().await.map_err(text)?;
            }
        }
        Ok(())
    })
}

/// The Secret Service's default collection, unlocked.
async fn session_keyring() -> Result<oo7::Keyring, String> {
    let service = oo7::dbus::Service::new().await.map_err(text)?;
    let keyring = oo7::Keyring::DBus(service.default_collection().await.map_err(text)?);
    keyring.unlock().await.map_err(text)?;
    Ok(keyring)
}

fn text(e: impl std::fmt::Display) -> String {
    e.to_string()
}

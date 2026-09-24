use std::path::PathBuf;

use gpui_kit::{Context, PathPromptOptions};

use super::CrowApp;

pub const FLEET_BACKGROUND: &str = "appearance.fleet_background";
pub const FLEET_BACKGROUND_OPACITY: &str = "appearance.fleet_background_opacity";
pub const FLEET_BACKGROUND_BLUR: &str = "appearance.fleet_background_blur";

/// The Fleet background as drawn: the picture for the current blur setting.
#[derive(Default)]
pub struct FleetBackground {
    /// (stored picture, blur) that `rendered` was made for.
    pub key: Option<(String, u32)>,
    pub rendered: Option<PathBuf>,
    pub pending: bool,
    pub error: Option<String>,
}

impl CrowApp {
    fn config_str(&self, row_id: &str) -> String {
        self.config.get_field(row_id).and_then(|f| f.value.as_str().map(str::to_string)).unwrap_or_default()
    }

    /// Opacity of the Fleet background, 0.0–1.0.
    pub fn fleet_background_opacity(&self) -> f32 {
        self.config_str(FLEET_BACKGROUND_OPACITY).parse::<f32>().unwrap_or(10.0).clamp(0.0, 100.0) / 100.0
    }

    /// The stored picture path, if one is set and still exists.
    pub fn fleet_background_source(&self) -> Option<PathBuf> {
        let p = self.config_str(FLEET_BACKGROUND);
        (!p.is_empty()).then(|| PathBuf::from(p)).filter(|p| p.exists())
    }

    /// Makes sure the picture for the current settings is rendered (blur is
    /// done once, off the UI thread, then cached on disk).
    pub fn ensure_fleet_background(&mut self, cx: &mut Context<Self>) {
        let key = self.fleet_background_source().map(|p| (p.to_string_lossy().into_owned(), self.config_str(FLEET_BACKGROUND_BLUR).parse::<u32>().unwrap_or(0)));
        if key == self.fleet_background.key || self.fleet_background.pending {
            return;
        }
        let Some((path, blur)) = key.clone() else {
            self.fleet_background = FleetBackground::default();
            return;
        };
        self.fleet_background.pending = true;
        cx.spawn(async move |entity, cx| {
            let result = cx.background_executor().spawn(async move { crate::appearance::rendered_background(std::path::Path::new(&path), blur) }).await;
            let _ = entity.update(cx, |this, cx| {
                let bg = &mut this.fleet_background;
                bg.pending = false;
                bg.key = key;
                match result {
                    Ok(p) => (bg.rendered, bg.error) = (Some(p), None),
                    Err(e) => (bg.rendered, bg.error) = (None, Some(e)),
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Opens the file picker; the chosen picture is copied into Crow's
    /// backgrounds folder and the setting saved.
    pub fn choose_fleet_background(&mut self, cx: &mut Context<Self>) {
        let picked = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false, prompt: Some("Choose a picture for the Fleet page".into()) });
        cx.spawn(async move |entity, cx| {
            let Ok(Ok(Some(paths))) = picked.await else { return };
            let Some(source) = paths.into_iter().next() else { return };
            let stored = cx.background_executor().spawn(async move { crate::appearance::store_background(&source) }).await;
            let _ = entity.update(cx, |this, cx| match stored {
                Ok(path) => {
                    this.update_config_field(FLEET_BACKGROUND, serde_json::Value::String(path.to_string_lossy().into_owned()), cx);
                    crate::appearance::prune_backgrounds(Some(&path));
                    this.fleet_background.error = None;
                }
                Err(e) => {
                    this.fleet_background.error = Some(e);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub fn remove_fleet_background(&mut self, cx: &mut Context<Self>) {
        self.update_config_field(FLEET_BACKGROUND, serde_json::Value::String(String::new()), cx);
        crate::appearance::prune_backgrounds(None);
    }
}

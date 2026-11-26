//! Cross-platform OS primitives shared across Crow (Windows 11, macOS, Linux).
//!
//! Centralizes platform differences for:
//! - Unified config/vault directory (`%APPDATA%\crow` on Windows, `~/.config/crow` on Unix)
//! - Current user & host identification (`USER` / `USERNAME`, `COMPUTERNAME`)
//! - Atomic file replacement (`MoveFileExW` on Windows, `std::fs::rename` on Unix)
//! - Suppressing console flicker (`CREATE_NO_WINDOW`) and attaching CLI consoles
//! - User-only NTFS DACL enforcement for SSH private keys, vault DB, and plugin secrets
//! - Safe URL opening (`ShellExecuteW` on Windows, `open` on macOS, `xdg-open` on Linux)

use std::path::{Path, PathBuf};
use std::process::Command;

/// Crow's unified configuration & state directory:
/// - Windows: `%APPDATA%\crow`
/// - Linux / macOS: `~/.config/crow` (also shared with Flatpak via `sandbox::share_native_folders`)
pub fn crow_config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
            .join("crow")
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = dirs::home_dir() {
            home.join(".config").join("crow")
        } else if let Some(dir) = dirs::config_dir() {
            dir.join("crow")
        } else {
            PathBuf::from(".crow")
        }
    }
}

/// Current local OS account name (`USER` on Unix, `USERNAME` on Windows).
pub fn current_username() -> Option<String> {
    for key in ["USER", "USERNAME", "LOGNAME"] {
        if let Ok(v) = std::env::var(key) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Current local machine hostname without spawning a shell where avoidable.
pub fn current_hostname() -> Option<String> {
    for path in ["/proc/sys/kernel/hostname", "/etc/hostname"] {
        if let Ok(text) = std::fs::read_to_string(path) {
            let h = text.trim();
            if !h.is_empty() {
                return Some(h.to_string());
            }
        }
    }
    if let Ok(name) = std::env::var("COMPUTERNAME") {
        let h = name.trim();
        if !h.is_empty() {
            return Some(h.to_string());
        }
    }
    let mut cmd = Command::new("hostname");
    configure_background_command(&mut cmd);
    cmd.output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
}

/// Marks a child command so it never flashes a console window on Windows.
pub fn configure_background_command(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = cmd;
}

/// Attaches to the parent console when invoked with CLI flags (`--check-plugin`,
/// `--check-language`, `--daemon`) on a Windows GUI-subsystem binary.
pub fn attach_parent_console() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Replaces `dst` with `src` atomically where the OS supports it.
/// On Windows, `std::fs::rename` fails when `dst` already exists, so use
/// `MoveFileExW(MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)`.
pub fn replace_file(src: &Path, dst: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        };
        let to_wide = |p: &Path| -> Vec<u16> {
            p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
        };
        let src_w = to_wide(src);
        let dst_w = to_wide(dst);
        // SAFETY: both pointers are valid NUL-terminated UTF-16 buffers.
        let ok = unsafe {
            MoveFileExW(
                src_w.as_ptr(),
                dst_w.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if ok != 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
    #[cfg(not(windows))]
    {
        std::fs::rename(src, dst)
    }
}

/// Restricts a file or directory on Windows via a protected NTFS DACL granting
/// access exclusively to the current process user SID (stripping inherited ACEs).
/// Required for Win32-OpenSSH private keys and Crow's vault/secrets files.
#[cfg(windows)]
pub fn restrict_to_current_user(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, ERROR_SUCCESS, HANDLE};
    use windows_sys::Win32::Security::Authorization::{
        SetEntriesInAclW, SetNamedSecurityInfoW, EXPLICIT_ACCESS_W, SET_ACCESS, SE_FILE_OBJECT,
        TRUSTEE_IS_SID, TRUSTEE_IS_USER, TRUSTEE_W,
    };
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenUser, ACL, DACL_SECURITY_INFORMATION, NO_INHERITANCE,
        PROTECTED_DACL_SECURITY_INFORMATION, SUB_CONTAINERS_AND_OBJECTS_INHERIT, TOKEN_QUERY,
        TOKEN_USER,
    };
    use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let is_dir = std::fs::symlink_metadata(path)?.is_dir();
    let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();

    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut needed: u32 = 0;
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
        if needed == 0 {
            let err = std::io::Error::last_os_error();
            CloseHandle(token);
            return Err(err);
        }

        let mut buf = vec![0u8; needed as usize];
        let ok = GetTokenInformation(
            token,
            TokenUser,
            buf.as_mut_ptr().cast(),
            needed,
            &mut needed,
        );
        CloseHandle(token);
        if ok == 0 {
            return Err(std::io::Error::last_os_error());
        }

        let token_user = &*(buf.as_ptr().cast::<TOKEN_USER>());
        let user_sid = token_user.User.Sid;

        let mut ea: EXPLICIT_ACCESS_W = std::mem::zeroed();
        ea.grfAccessPermissions = FILE_ALL_ACCESS;
        ea.grfAccessMode = SET_ACCESS;
        ea.grfInheritance = if is_dir {
            SUB_CONTAINERS_AND_OBJECTS_INHERIT
        } else {
            NO_INHERITANCE
        };
        ea.Trustee = TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: 0,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: user_sid.cast(),
        };

        let mut new_dacl: *mut ACL = std::ptr::null_mut();
        let status = SetEntriesInAclW(1, &ea, std::ptr::null(), &mut new_dacl);
        if status != ERROR_SUCCESS {
            return Err(std::io::Error::from_raw_os_error(status as i32));
        }

        let sec_status = SetNamedSecurityInfoW(
            wide_path.as_mut_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            new_dacl,
            std::ptr::null_mut(),
        );
        LocalFree(new_dacl.cast());

        if sec_status != ERROR_SUCCESS {
            return Err(std::io::Error::from_raw_os_error(sec_status as i32));
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn restrict_to_current_user(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Opens an HTTP/HTTPS URL in the user's default browser without invoking a shell.
pub fn open_url(url: &str) {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return;
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let op: Vec<u16> = "open\0".encode_utf16().collect();
        let target: Vec<u16> = std::ffi::OsStr::new(url)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        // SAFETY: valid NUL-terminated UTF-16 strings; scheme validated above.
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                op.as_ptr(),
                target.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
    #[cfg(not(windows))]
    {
        #[cfg(target_os = "macos")]
        let opener = "open";
        #[cfg(not(target_os = "macos"))]
        let opener = "xdg-open";
        let _ = Command::new(opener)
            .arg(url)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_and_username_resolve() {
        let dir = crow_config_dir();
        assert_eq!(dir.file_name().and_then(|s| s.to_str()), Some("crow"));
        assert!(current_username().is_some());
    }

    #[cfg(windows)]
    #[test]
    fn restrict_to_current_user_applies_protected_ntfs_dacl() {
        let dir = std::env::temp_dir().join(format!("crow-acl-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("id_test");
        std::fs::write(&file, "secret").unwrap();

        restrict_to_current_user(&file).expect("protected DACL applied");
        restrict_to_current_user(&dir).expect("protected dir DACL applied");

        // Verify via icacls that BUILTIN\Users / Everyone are not present and
        // inheritance is stripped (no `(I)` inherited ACE markers).
        let mut cmd = Command::new("icacls");
        configure_background_command(&mut cmd);
        let out = cmd.arg(&file).output().expect("icacls runs");
        assert!(out.status.success());
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(!text.contains("(I)"), "inherited ACEs must be stripped: {text}");
        assert!(!text.contains("BUILTIN\\Users"), "Users group must not have access: {text}");
        assert!(!text.contains("Everyone"), "Everyone must not have access: {text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

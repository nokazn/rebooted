use std::process::Command;

use crate::error::{Error, Result};

pub fn reboot() -> Result<()> {
    #[cfg(target_os = "macos")]
    return reboot_macos();

    #[cfg(target_os = "linux")]
    return reboot_linux();

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    compile_error!("Unsupported OS: macOS or Linux required");
}

#[cfg(target_os = "macos")]
fn reboot_macos() -> Result<()> {
    // osascript does not require sudo, unlike `shutdown`
    let status = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to restart"])
        .status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            Command::new("shutdown")
                .args(["-r", "now"])
                .spawn()
                .map_err(|e| {
                    Error::RebootFailed(format!(
                        "both osascript and shutdown failed: {e}\n\
                         Run 'sudo shutdown -r now' to reboot manually"
                    ))
                })?;
            Ok(())
        }
    }
}

#[cfg(target_os = "linux")]
fn reboot_linux() -> Result<()> {
    let status = Command::new("systemctl").arg("reboot").status();

    match status {
        Ok(s) if s.success() => Ok(()),
        _ => {
            Command::new("shutdown")
                .args(["-r", "now"])
                .spawn()
                .map_err(|e| {
                    Error::RebootFailed(format!(
                        "both systemctl reboot and shutdown failed: {e}\n\
                         Run 'sudo shutdown -r now' to reboot manually"
                    ))
                })?;
            Ok(())
        }
    }
}

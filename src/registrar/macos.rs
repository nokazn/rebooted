use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};

use super::{Registrar, ServiceSpec};

pub struct LaunchAgentRegistrar;

impl LaunchAgentRegistrar {
    fn launch_agents_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
        Ok(home.join("Library/LaunchAgents"))
    }

    fn plist_path(label: &str) -> Result<PathBuf> {
        Ok(Self::launch_agents_dir()?.join(format!("com.rebooted.{label}.plist")))
    }

    fn render_plist(spec: &ServiceSpec) -> String {
        let program = spec.program.display().to_string();
        let args_xml: String = spec
            .args
            .iter()
            .map(|a| format!("        <string>{}</string>\n", xml_escape(a)))
            .collect();
        let log_out = format!("/tmp/rebooted-{}.log", spec.label);
        let log_err = format!("/tmp/rebooted-{}-err.log", spec.label);

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.rebooted.{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{program}</string>
{args_xml}    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log_out}</string>
    <key>StandardErrorPath</key>
    <string>{log_err}</string>
</dict>
</plist>
"#,
            label = spec.label,
            program = xml_escape(&program),
            args_xml = args_xml,
            log_out = log_out,
            log_err = log_err,
        )
    }
}

impl Registrar for LaunchAgentRegistrar {
    fn register(&self, spec: &ServiceSpec) -> Result<()> {
        let dir = Self::launch_agents_dir()?;
        std::fs::create_dir_all(&dir)?;

        let plist_path = Self::plist_path(&spec.label)?;
        let content = Self::render_plist(spec);
        std::fs::write(&plist_path, content)?;

        eprintln!("Registered LaunchAgent: {}", plist_path.display());
        Ok(())
    }

    fn unregister(&self, label: &str) -> Result<()> {
        let plist_path = Self::plist_path(label)?;

        // may not be loaded yet; ignore the error
        let _ = Command::new("launchctl")
            .args(["unload", &plist_path.to_string_lossy()])
            .status();

        if plist_path.exists() {
            std::fs::remove_file(&plist_path).map_err(|e| {
                Error::UnregistrationFailed(format!(
                    "failed to remove {}: {e}",
                    plist_path.display()
                ))
            })?;
        }
        Ok(())
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

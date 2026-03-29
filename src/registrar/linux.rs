use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};

use super::{Registrar, ServiceSpec};

pub struct SystemdRegistrar;

impl SystemdRegistrar {
    fn user_service_dir() -> Result<PathBuf> {
        let config = dirs::config_dir().ok_or(Error::HomeDirNotFound)?;
        Ok(config.join("systemd/user"))
    }

    fn unit_path(label: &str) -> Result<PathBuf> {
        Ok(Self::user_service_dir()?.join(format!("rebooted-{label}.service")))
    }

    fn render_unit(spec: &ServiceSpec) -> String {
        let program = spec.program.display().to_string();
        let args: Vec<String> = spec.args.iter().map(|a| shell_quote(a)).collect();
        let exec_start = format!("{} {}", shell_quote(&program), args.join(" "));

        format!(
            "[Unit]\nDescription=rebooted one-shot: {label}\nAfter=default.target\n\n[Service]\nType=oneshot\nExecStart={exec_start}\nStandardOutput=journal\nStandardError=journal\n\n[Install]\nWantedBy=default.target\n",
            label = spec.label,
            exec_start = exec_start,
        )
    }

    fn systemctl_user(args: &[&str]) -> Result<()> {
        let status = Command::new("systemctl")
            .arg("--user")
            .args(args)
            .status()
            .map_err(|e| Error::RegistrationFailed(format!("failed to run systemctl: {e}")))?;
        if !status.success() {
            return Err(Error::RegistrationFailed(format!(
                "systemctl --user {} failed (exit: {})",
                args.join(" "),
                status
            )));
        }
        Ok(())
    }
}

impl Registrar for SystemdRegistrar {
    fn register(&self, spec: &ServiceSpec) -> Result<()> {
        let dir = Self::user_service_dir()?;
        std::fs::create_dir_all(&dir)?;

        let unit_path = Self::unit_path(&spec.label)?;
        std::fs::write(&unit_path, Self::render_unit(spec))?;

        Self::systemctl_user(&["daemon-reload"])?;
        Self::systemctl_user(&["enable", &format!("rebooted-{}.service", spec.label)])?;

        eprintln!("Registered systemd user service: {}", unit_path.display());
        Ok(())
    }

    fn unregister(&self, label: &str) -> Result<()> {
        let service_name = format!("rebooted-{label}.service");

        // may not be enabled yet; ignore the error
        let _ = Command::new("systemctl")
            .args(["--user", "disable", &service_name])
            .status();

        let unit_path = Self::unit_path(label)?;
        if unit_path.exists() {
            std::fs::remove_file(&unit_path).map_err(|e| {
                Error::UnregistrationFailed(format!(
                    "failed to remove {}: {e}",
                    unit_path.display()
                ))
            })?;
        }

        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        Ok(())
    }
}

pub struct CrontabRegistrar;

impl CrontabRegistrar {
    fn entry_marker(label: &str) -> String {
        format!("# rebooted:{label}")
    }

    fn current_crontab() -> Result<String> {
        let output = Command::new("crontab").arg("-l").output()?;
        // crontab -l exits with 1 when the crontab is empty; treat that as empty
        if output.status.success() || output.stdout.is_empty() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Ok(String::new())
        }
    }

    fn write_crontab(content: &str) -> Result<()> {
        use std::io::Write;
        let mut child = Command::new("crontab")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(content.as_bytes())?;
        let status = child.wait()?;
        if !status.success() {
            return Err(Error::RegistrationFailed(
                "failed to write crontab".to_string(),
            ));
        }
        Ok(())
    }
}

impl Registrar for CrontabRegistrar {
    fn register(&self, spec: &ServiceSpec) -> Result<()> {
        let program = spec.program.display().to_string();
        let args: Vec<String> = spec.args.iter().map(|a| shell_quote(a)).collect();
        let cmd = format!("{} {}", shell_quote(&program), args.join(" "));
        let marker = Self::entry_marker(&spec.label);
        let entry = format!("@reboot {cmd} {marker}\n");

        let mut crontab = Self::current_crontab()?;
        crontab.push_str(&entry);
        Self::write_crontab(&crontab)?;

        eprintln!("Added @reboot crontab entry (label: {})", spec.label);
        Ok(())
    }

    fn unregister(&self, label: &str) -> Result<()> {
        let marker = Self::entry_marker(label);
        let crontab = Self::current_crontab()?;
        let filtered: String = crontab
            .lines()
            .filter(|l| !l.contains(&marker))
            .map(|l| format!("{l}\n"))
            .collect();
        Self::write_crontab(&filtered)?;
        Ok(())
    }
}

enum Backend {
    Systemd(SystemdRegistrar),
    Crontab(CrontabRegistrar),
}

impl Registrar for Backend {
    fn register(&self, spec: &ServiceSpec) -> Result<()> {
        match self {
            Backend::Systemd(r) => r.register(spec),
            Backend::Crontab(r) => r.register(spec),
        }
    }

    fn unregister(&self, label: &str) -> Result<()> {
        match self {
            Backend::Systemd(r) => r.unregister(label),
            Backend::Crontab(r) => r.unregister(label),
        }
    }
}

pub fn new_registrar() -> Box<dyn Registrar> {
    if is_systemd_user_available() {
        Box::new(Backend::Systemd(SystemdRegistrar))
    } else {
        eprintln!("systemd user session unavailable; falling back to crontab");
        Box::new(Backend::Crontab(CrontabRegistrar))
    }
}

fn is_systemd_user_available() -> bool {
    Command::new("systemctl")
        .args(["--user", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':' | '='))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

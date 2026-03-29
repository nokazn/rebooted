mod cli;
mod error;
mod reboot;
mod registrar;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use clap::Parser;

use cli::Cli;
use error::{Error, Result};
use registrar::{new_registrar, ServiceSpec};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run(cli).map_err(|e| anyhow::anyhow!("{e}"))
}

fn run(cli: Cli) -> Result<()> {
    if let Some(label) = cli.internal_exec {
        return exec_and_cleanup(&label, &cli.command);
    }

    if cli.command.is_empty() {
        return Err(Error::NoCommandSpecified);
    }

    let label = cli.label.unwrap_or_else(|| generate_label(&cli.command));
    let program = resolve_self()?;
    let spec = build_spec(label.clone(), program, &cli.command);
    let registrar = new_registrar();

    registrar.register(&spec)?;
    eprintln!("Registered command to run after reboot: {:?}", cli.command);

    if cli.dry_run {
        eprintln!("--dry-run: skipping reboot");
        return Ok(());
    }

    eprintln!("Rebooting...");
    reboot::reboot()
}

fn exec_and_cleanup(label: &str, command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err(Error::NoCommandSpecified);
    }

    // unregister before exec so the service won't run again even if the command fails
    let registrar = new_registrar();
    if let Err(e) = registrar.unregister(label) {
        eprintln!("warning: failed to unregister service: {e}");
    }

    let err = Command::new(&command[0]).args(&command[1..]).exec();
    Err(Error::Io(err))
}

fn generate_label(command: &[String]) -> String {
    let mut hasher = DefaultHasher::new();
    command.hash(&mut hasher);
    let hash = hasher.finish();
    format!("{hash:016x}")[..8].to_string()
}

fn resolve_self() -> Result<PathBuf> {
    std::env::current_exe()
        .map_err(|e| Error::RegistrationFailed(format!("failed to resolve binary path: {e}")))
}

fn build_spec(label: String, program: PathBuf, command: &[String]) -> ServiceSpec {
    let mut args = vec![
        "--internal-exec".to_string(),
        label.clone(),
        "--".to_string(),
    ];
    args.extend_from_slice(command);
    ServiceSpec {
        label,
        program,
        args,
    }
}

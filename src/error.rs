use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No command specified. Use the form: rebooted -- <command>")]
    NoCommandSpecified,

    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Failed to register service: {0}")]
    RegistrationFailed(String),

    #[error("Failed to unregister service: {0}")]
    UnregistrationFailed(String),

    #[error("Reboot failed: {0}\nRun 'sudo shutdown -r now' to reboot manually")]
    RebootFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

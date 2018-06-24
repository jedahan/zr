use std::ffi::OsString;
use std::fmt;
use std::write;

#[derive(Debug)]
pub enum Error {
    EnvironmentVariableNotUnicode { key: String, value: OsString },
    InvalidPluginName { plugin_name: String },
    Clap(clap::Error),
    Io(std::io::Error),
    Git(git2::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            EnvironmentVariableNotUnicode { key, value } => write!(
                f,
                "The value in the environment variable '{}' is not utf-8: {}",
                key,
                value.to_string_lossy()
            ),
            InvalidPluginName { plugin_name } => write!(
                f,
                "The plugin name must be formatted 'author/name', found '{}'",
                plugin_name
            ),
            Clap(error) => write!(f, "Clap error: {}", error.to_string()),
            Io(error) => write!(f, "Io error: {}", error.to_string()),
            Git(error) => write!(f, "Git error: {}", error.to_string()),
        }
    }
}

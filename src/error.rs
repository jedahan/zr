extern crate git2;
extern crate clap;
extern crate url;

use std::fmt;
use std::io;
use std::ffi::OsString;

pub enum Error {
    EnvironmentVariableNotUnicode { key: String, value: OsString },
    InvalidPluginName { plugin_name: String },
    InvalidIdentifier { id: String },
    Clap(clap::Error),
    Io(io::Error),
    Git(git2::Error),
    Url(url::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            EnvironmentVariableNotUnicode {ref key, ref value} =>
                write!(f, "The value in the environment variable '{}' is not utf-8: {}", key, value.to_string_lossy()),
            InvalidPluginName {ref plugin_name} =>
                write!(f, "The plugin name must be formatted 'author/name', found '{}'", plugin_name),
            InvalidIdentifier {ref id} =>
                write!(f, "The host/author/name identifiers can only contain [a-zA-Z0-9.-], found '{}'", id),
            Clap(ref error) =>
                write!(f, "Clap error: {}", error.to_string()),
            Url(ref error) =>
                write!(f, "Url parse error: {}", error.to_string()),
            Io(ref error) =>
                write!(f, "Io error: {}", error.to_string()),
            Git(ref error) =>
                write!(f, "Git error: {}", error.to_string()),
        }
    }
}

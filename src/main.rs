#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate git2;
extern crate libc;

use std::fmt;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::ffi::{OsStr,OsString};
use git2::Repository;
use std::iter::FromIterator;

/// eprintln! will be in stable soon
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        extern crate std;
        use std::io::prelude::*;
        if let Err(result) = writeln!(&mut std::io::stderr(), $($arg)*) {
            panic!(result);
        }
    }}
}

struct Plugin {
    author: String,
    name: String,
    files: HashSet<PathBuf>
}

enum Error {
    EnvironmentVariableNotUnicode { key: String, value: OsString },
    InvalidPluginName { plugin_name: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            EnvironmentVariableNotUnicode {ref key, ref value} =>
                write!(f, "The value in the environment variable '{}' is not utf-8: {}", key, value.to_string_lossy()),
            InvalidPluginName {ref plugin_name} =>
                write!(f, "The plugin name must be formatted 'author/name', found '{}'", plugin_name),
        }
    }
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut basedirs = HashSet::new();
        writeln!(f, "# {}/{}", self.author, self.name)?;
        for file in &self.files {
            if let Some(basedir) = file.parent() {
                basedirs.insert(basedir);
            }
            writeln!(f, "source {}", file.display())?;
        }
        for basedir in basedirs {
            writeln!(f, "fpath+={}/", basedir.display())?;
            writeln!(f, "PATH={}:$PATH", basedir.display())?;
        }
        Ok(())
    }
}

fn split(plugin_name: &str) -> Result<(String, String), Error> {
    let plugin_path = PathBuf::from(plugin_name);
    if plugin_path.components().count() != 2 {
        return Err(Error::InvalidPluginName { plugin_name: plugin_name.to_string() })
    }

    let name = plugin_path.components().last().unwrap().as_os_str().to_str().unwrap();
    let author = plugin_path.parent().unwrap().components().last().unwrap().as_os_str().to_str().unwrap();

    Ok((author.to_string(), name.to_string()))
}

impl Plugin {
    pub fn from_plugin_name(zr_init: &Path, plugin_name: &str) -> Result<Plugin, Error> {
        let (author, name) = split(plugin_name)?;
        Plugin::new(&zr_init, &author, &name)
    }

    pub fn new(zr_init: &Path, author: &str, name: &str) -> Result<Plugin, Error> {
        let path = zr_init.parent().unwrap().join("plugins").join(&author).join(&name);
        if ! path.exists() {
            fs::create_dir(path.parent().unwrap()).expect("Had trouble creating a directory");
            let url = format!("https://github.com/{}/{}", author, name);
            let _ = match Repository::clone(&url, &path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            };
        }

        let files: Vec<PathBuf> = path.read_dir().unwrap()
            .filter_map(std::result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        let sources: Vec<PathBuf> = {
            if let Some(antigen_plugin_file) = files.iter().find(|&file| *file == path.join(&name).with_extension("plugin.zsh")) {
                vec![antigen_plugin_file.to_owned()]
            } else if let Some(prezto_plugin_file) = files.iter().find(|&file| *file == path.join("init.zsh")) {
                vec![prezto_plugin_file.to_owned()]
            } else {
                let zsh_plugin_files: Vec<_> = files.iter().cloned().filter(|ref file| file.extension() == Some(OsStr::new("zsh"))).collect();
                let shell_files = if zsh_plugin_files.is_empty() {
                    files.iter().cloned().filter(|file| file.extension().unwrap() == "sh").collect()
                } else {
                    zsh_plugin_files
                };
                shell_files
            }

        };

        Ok(Plugin { author: author.to_string(), name: name.to_string(), files: HashSet::from_iter(sources) } )
    }

    pub fn from_files(zr_init: &Path, author: &str, name: &str, files: Vec<PathBuf>) -> Plugin {
        let path = zr_init.parent().unwrap().join("plugins").join(&author).join(&name);
        let mapped = files.iter().cloned().map(|file| path.join(&file)).collect();

        Plugin {
            author: author.to_string(),
            name: name.to_string(),
            files: mapped,
        }
    }

}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        std::process::exit(libc::EXIT_FAILURE);
    }
}

fn get_var(key: &str) -> Result<Option<String>, Error> {
    use std::env::VarError::*;

    match std::env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(NotPresent) => Ok(None),
        Err(NotUnicode(value)) => Err(Error::EnvironmentVariableNotUnicode { key: key.to_string(), value: value} ),
    }
}

#[cfg(test)]
mod tests {
    pub fn test_not_utf8_environment_variables_error_out() {
        let bad_byte = b"\x192";
        std::env::set_var("ZR_HOME", bad_byte);
    }
}

fn run() -> Result<(), Error> {
    let zr_home = get_var("ZR_HOME")?;

    let home = get_var("HOME")?;
    let default_home = format!("{}/.zr", home.unwrap());

    let zr_init = PathBuf::from(zr_home.unwrap_or(default_home)).join("init.zsh");

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "zsh plugin manager")
        (@subcommand reset => (about: "delete init file") )
        (@subcommand debug => (about: "print debug information") )
        (@subcommand list => (about: "list plugins") )
        (@subcommand add =>
            (about: "add plugin to init file")
            (@arg plugin: +required "plugin/name")
            (@arg file: "optional/path/to/file.zsh")
        )
    );

    match zr.clone().get_matches().subcommand() {
        ("add", Some(matches)) => {
            let plugin_name = matches.value_of("plugin").expect("add should have a plugin specified");
            let (author, name) = split(&plugin_name)?;
            if let Some(file) = matches.value_of("file") {
                add_file(&zr_init, author, name, file)?;
            } else {
                add(&zr_init, author, name)?;
            }
        },
        ("list", _) => list(&zr_init),
        ("reset", _) => reset(&zr_init),
        (_, _) => zr.print_help().unwrap(),
    }

    Ok(())
}

fn reset(zr_init: &Path) {
    if let Err(error) = fs::remove_file(zr_init) {
        if error.kind() != ErrorKind::NotFound {
            Err(error).unwrap()
        }
    }
}

fn list(zr_init: &Path) {
    for plugin in plugins(zr_init) {
        println!("{}/{}", plugin.author, plugin.name);
    }
}

fn plugins(zr_init: &Path) -> Vec<Plugin> {
    if ! zr_init.exists() {
        return vec![];
    }

    let init_file = OpenOptions::new().read(true).open(&zr_init).unwrap();

    BufReader::new(&init_file)
        .lines()
        .map(|line| line.unwrap())
        .filter(|line| line.starts_with('#'))
        .map(|line| line.split_whitespace().last().unwrap().to_owned())
        .map(|plugin_name| Plugin::from_plugin_name(&zr_init, &plugin_name).ok().unwrap())
        .collect::<Vec<Plugin>>()
}

fn add_file(zr_init: &Path, author: String, name: String, file: &str) -> Result<(), Error> {
    let mut plugins = plugins(zr_init);
    if plugins.iter().find(|ref plugin| (&plugin.name, &plugin.author) == (&name, &author)).is_none() {
        plugins.push(Plugin::from_files(&zr_init, &author, &name, vec![PathBuf::from(&file)]));
    } else if let Some(plugin) = plugins.iter_mut().find(|ref plugin| (&plugin.name, &plugin.author) == (&name, &author)) {
        plugin.files.insert(zr_init.parent().unwrap().join("plugins").join(&author).join(&name).join(&file));
    }

    save(zr_init, plugins)
}

fn add(zr_init: &Path, author: String, name: String) -> Result<(), Error> {
    let mut plugins = plugins(zr_init);

    if plugins.iter().all(|ref plugin| (&plugin.name, &plugin.author) != (&name, &author)) {
        plugins.push(Plugin::new(&zr_init, &author, &name)?);
    }

    save(zr_init, plugins)
}

fn save(zr_init: &Path, plugins: Vec<Plugin>) -> Result<(), Error> {
    let temp_filename = format!("{}init.zsh", std::env::temp_dir().display());
    let mut temp_file = OpenOptions::new().write(true).create_new(true).open(&temp_filename).unwrap();

    for plugin in plugins {
        writeln!(temp_file, "{}", plugin)
            .expect("Should be able to write to temp_file");
    }
    writeln!(temp_file, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
        .expect("Should be able to write the autoload line");

    fs::rename(&temp_filename, &zr_init).unwrap();
    Ok(())
}

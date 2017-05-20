#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate git2;
extern crate libc;

use std::{fmt, fs};
use std::path::{Path, PathBuf};
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

struct Plugins {
    home: PathBuf,
    plugins: Vec<Plugin>
}

impl Plugins {
    pub fn reset(&self) -> Result<(), Error> {
        let filepath = self.home.join("init.zsh");
        fs::remove_file(&filepath).or_else(|error|
             if error.kind() == ErrorKind::NotFound {
                 Ok(())
             } else {
                 Err(Error::IoError(error))
             })
    }

    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let plugin_home = self.home.join("plugins").join(&plugin.author).join(&plugin.name);
            if let Ok(repo) = git2::Repository::open(&plugin_home) {
                if let Ok(remotes) = repo.remotes() {
                    if let Some(first_remote) = remotes.get(0) {
                        let mut cb = git2::RemoteCallbacks::new();
                        cb.update_tips(|_, a, b| {
                            if ! a.is_zero() {
                                println!("updated {}/{} from {:6}..{:6}", &plugin.author, &plugin.name, a, b);
                            }
                            true
                        });
                        let mut opts = git2::FetchOptions::new();
                        opts.remote_callbacks(cb);
                        let mut remote = repo.find_remote(first_remote).unwrap();
                        let refspec = "refs/heads/*:refs/heads/*";
                        remote.fetch(&[refspec], Some(&mut opts), None)
                            .map_err(|error| Error::GitError(error))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn new(zr_home: PathBuf) -> Plugins {
        let zr_init = &zr_home.join("init.zsh");
        Plugins {
            home: zr_home.clone(),
            plugins: if ! zr_init.exists() { vec![] } else {
                let init_file = OpenOptions::new().read(true).open(&zr_init).unwrap();

                BufReader::new(&init_file)
                    .lines()
                    .map(|line| line.unwrap())
                    .filter(|line| line.starts_with('#'))
                    .map(|line| line.split_whitespace().last().unwrap().to_owned())
                    .map(|plugin_name| Plugin::from_plugin_name(&zr_home, &plugin_name).ok().unwrap())
                    .collect::<Vec<Plugin>>()
            }
        }
    }

    pub fn list(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            println!("{}/{}", plugin.author, plugin.name)
        }
        Ok(())
    }

    pub fn add(&mut self, plugin_name: &str, file: Option<&str>) -> Result<(), Error> {
        let (author, name) = split(&plugin_name)?;
        if let Some(filepath) = file {
            if self.plugins.iter().find(|ref plugin| (&plugin.name, &plugin.author) == (&name, &author)).is_none() {
                let files = vec![PathBuf::from(&filepath)];
                let plugin = Plugin::from_files(&self.home, &author, &name, files);
                self.plugins.push(plugin);
            } else if let Some(plugin) = self.plugins.iter_mut().find(|ref plugin| (&plugin.name, &plugin.author) == (&name, &author)) {
                let file = self.home.join("plugins").join(&author).join(&name).join(&filepath);
                plugin.files.insert(file);
            }
        } else {
            if self.plugins.iter().all(|ref plugin| (&plugin.name, &plugin.author) != (&name, &author)) {
                let plugin = Plugin::new(&self.home, &author, &name)?;
                self.plugins.push(plugin);
            }
        }

        let temp_filename = format!("{}init.zsh", std::env::temp_dir().display());
        let mut temp_file = OpenOptions::new().write(true).create_new(true).open(&temp_filename).unwrap();

        for plugin in &self.plugins {
            writeln!(temp_file, "{}", plugin)
                .expect("Should be able to write to temp_file");
        }
        writeln!(temp_file, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
            .expect("Should be able to write the autoload line");

        fs::rename(&temp_filename, &self.home.join("init.zsh")).unwrap();
        Ok(())
    }
}

struct Plugin {
    author: String,
    name: String,
    files: HashSet<PathBuf>
}

enum Error {
    EnvironmentVariableNotUnicode { key: String, value: OsString },
    InvalidPluginName { plugin_name: String },
    ClapError(clap::Error),
    IoError(std::io::Error),
    GitError(git2::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            EnvironmentVariableNotUnicode {ref key, ref value} =>
                write!(f, "The value in the environment variable '{}' is not utf-8: {}", key, value.to_string_lossy()),
            InvalidPluginName {ref plugin_name} =>
                write!(f, "The plugin name must be formatted 'author/name', found '{}'", plugin_name),
            ClapError(ref error) =>
                write!(f, "Clap error: {}", error.to_string()),
            IoError(ref error) =>
                write!(f, "Io error: {}", error.to_string()),
            GitError(ref error) =>
                write!(f, "Git error: {}", error.to_string()),
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
    pub fn from_plugin_name(zr_home: &Path, plugin_name: &str) -> Result<Plugin, Error> {
        let (author, name) = split(plugin_name)?;
        Plugin::new(&zr_home, &author, &name)
    }

    pub fn new(zr_home: &Path, author: &str, name: &str) -> Result<Plugin, Error> {
        let path = zr_home.join("plugins").join(&author).join(&name);
        if ! path.is_dir() {
            fs::create_dir(path.parent().unwrap()).map_err(|error| Error::IoError(error))?;
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

    pub fn from_files(zr_home: &Path, author: &str, name: &str, files: Vec<PathBuf>) -> Plugin {
        let path = zr_home.join("plugins").join(&author).join(&name);
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

fn run() -> Result<(), Error> {
    let zr_home = get_var("ZR_HOME")?;
    let home = get_var("HOME")?;
    let default_home = format!("{}/.zr", home.unwrap());

    let mut plugins = Plugins::new(PathBuf::from(zr_home.unwrap_or(default_home)));

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "z:rat: - zsh plugin manager")
        (@subcommand reset => (about: "delete init file") )
        (@subcommand debug => (about: "print debug information") )
        (@subcommand list => (about: "list plugins") )
        (@subcommand update => (about: "update plugins") )
        (@subcommand add =>
            (about: "add plugin to init file")
            (@arg plugin: +required "plugin/name")
            (@arg file: "optional/path/to/file.zsh")
        )
    );

    match zr.clone().get_matches().subcommand() {
        ("add", Some(m)) => plugins.add(m.value_of("plugin").unwrap(), m.value_of("file")),
        ("list", _) => plugins.list(),
        ("reset", _) => plugins.reset(),
        ("update", _) => plugins.update(),
        (_, _) => zr.print_help().map_err(Error::ClapError),
    }
}

#[cfg(test)]
mod tests {
    pub fn test_not_utf8_environment_variables_error_out() {
        let bad_byte = b"\x192";
        std::env::set_var("ZR_HOME", bad_byte);
        run()
    }

    pub fn test_plugin_name_must_have_author() {
        let plugin_name = "test_plugin";
        add();
    }
}


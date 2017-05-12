#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate git2;
extern crate libc;

/// The prelude module makes it easy to split this file into multiple files
mod prelude {
    pub use std::fmt;
    pub use std::path::{Path, PathBuf};
    pub use std::fs;
    pub use std::io::{BufRead, BufReader, ErrorKind, Write};
    pub use std::collections::HashSet;
    pub use std::fs::OpenOptions;
    pub use std::ffi::{OsStr,OsString};
    pub use git2::Repository;
}

use prelude::*;

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

/// A plugin is just a simple name (like "geometry"), and a list of files to load
struct Plugin {
    name: String,
    files: HashSet<PathBuf>
}

enum Error {
    EnvironmentVariableNotUnicode { key: String, value: OsString }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            EnvironmentVariableNotUnicode {ref key, ref value} =>
                write!(f, "The value in the environment variable '{}' is not utf-8: {}", key, value.to_string_lossy()),
        }
    }
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut basedirs = HashSet::new();
        writeln!(f, "# {}", self.name)?;
        for file in &self.files {
            basedirs.insert(file.parent().unwrap());
            writeln!(f, "source {}", file.display())?;
        }
        for basedir in basedirs {
            writeln!(f, "fpath+={}/", basedir.display())?;
            writeln!(f, "PATH={}:$PATH", basedir.display())?;
        }
        Ok(())
    }
}

impl Plugin {

    pub fn from_name(zr_init: &Path, name: &str) -> Plugin {
        let path = zr_init.parent().unwrap().join("plugins").join(&name);
        if ! path.exists() {
            let url = format!("https://github.com/{}", name);
            let _ = match Repository::clone(&url, &path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to clone: {}", e),
            };
        }
        Plugin::from_path(&path)
    }

    pub fn from_files(name: &Path, files: Vec<PathBuf>) -> Plugin {
        Plugin {
            name: name.to_str().unwrap().to_string(),
            files: files.iter().cloned().collect(),
        }
    }

    pub fn from_path(path: &Path) -> Plugin {
        let name = path.parent().unwrap();

        let files: Vec<_> = path.read_dir().unwrap()
            .filter_map(std::result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        // antigen style
        if let Some(antigen_plugin_file) = files.iter().find(|&file| *file == path.join(name).with_extension("plugin.zsh")) {
            return Plugin::from_files(name, vec![antigen_plugin_file.to_owned()]);
        }

        // prezto style
        if let Some(prezto_plugin_file) = files.iter().find(|&file| *file == path.join("init.zsh")) {
            return Plugin::from_files(name, vec![prezto_plugin_file.to_owned()]);
        }

        // zsh plugins
        let zsh_plugin_files: Vec<_> = files.iter().cloned().filter(|ref file| file.extension() == Some(OsStr::new("zsh"))).collect();
        if ! zsh_plugin_files.is_empty() {
            return Plugin::from_files(name, zsh_plugin_files);
        }

        // sh plugins
        let sh_plugin_files: Vec<_> = files.iter().cloned().filter(|file| file.extension().unwrap() == "sh").collect();
        if ! sh_plugin_files.is_empty() {
            return Plugin::from_files(name, sh_plugin_files);
        }

        Plugin::from_files(name, vec![])
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
            let plugin = matches.value_of("plugin").expect("add should have a plugin specified");
            if matches.is_present("file") {
                add(&zr_init, plugin, matches.value_of("file").unwrap())
            } else {
                add(&zr_init, plugin, "")
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
        println!("{}", plugin.name);
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
        .map(|plugin_name| Plugin::from_name(zr_init, &plugin_name))
        .collect::<Vec<Plugin>>()
}

fn add(zr_init: &Path, name: &str, file: &str) {
    let plugins = load(zr_init, name, file);
    save(zr_init, plugins);
}

fn load(zr_init: &Path, name: &str, file: &str) -> Vec<Plugin> {
    let mut plugins = plugins(zr_init);

    let plugin_exists = plugins.iter().any(|plugin| plugin.name == name);
    let has_filename = file != "";

    if has_filename {
        if plugin_exists {
            plugins.iter_mut().find(|plugin| plugin.name == name).unwrap().files.insert(PathBuf::from(&file));
        } else {
            plugins.push(Plugin::from_files(Path::new(name), vec![PathBuf::from(&file)]));
        }
    } else if !plugin_exists {
        plugins.push(Plugin::from_name(zr_init, name));
    }

    plugins
}

fn save(zr_init: &Path, plugins: Vec<Plugin>) {
    let temp_filename = format!("{}init.zsh", std::env::temp_dir().display());
    let mut temp_file = OpenOptions::new().write(true).create_new(true).open(&temp_filename).unwrap();

    for plugin in plugins {
        writeln!(temp_file, "{}", plugin)
            .expect("Should be able to write plugins");
    }
    writeln!(temp_file, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
        .expect("Should be able to write the autoload line");

    fs::rename(&temp_filename, &zr_init).unwrap();
}

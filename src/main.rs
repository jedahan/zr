#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

use std::fmt;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::collections::HashSet;
use std::fs::OpenOptions;

#[macro_use]
extern crate clap;

struct Plugin {
    name: String,
    files: HashSet<PathBuf>
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
    pub fn new_from_files(name: &str, files: Vec<PathBuf>) -> Plugin {
        Plugin {
            name: name.to_owned(),
            files: files.iter().cloned().collect(),
        }
    }

    pub fn new(init_home: &Path, name: &str) -> Plugin {
        let path = init_home.parent().unwrap().join("plugins").join(&name);
        let pathname = PathBuf::from(name);
        let shortname = pathname.file_name().unwrap().to_string_lossy();

        let files: Vec<_> = path.read_dir().unwrap()
            .filter_map(std::result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        // antigen style
        if let Some(antigen_plugin_file) = files.iter().find(|file| file.file_name().unwrap().to_string_lossy() == format!("{}.plugin.zsh", &shortname)) {
            return Self::new_from_files(name, vec![antigen_plugin_file.to_owned()]);
        }

        // prezto style
        if let Some(prezto_plugin_file) = files.iter().find(|file| file.file_name().unwrap() == path.join("init.zsh")) {
            return Self::new_from_files(name, vec![prezto_plugin_file.to_owned()]);
        }

        // zsh plugins
        let zsh_plugin_files: Vec<_> = files.iter().cloned().filter(|file| file.extension().unwrap() == "zsh").collect();
        if ! zsh_plugin_files.is_empty() {
            return Self::new_from_files(name, zsh_plugin_files);
        }

        // sh plugins
        let sh_plugin_files: Vec<_> = files.iter().cloned().filter(|file| file.extension().unwrap() == "sh").collect();
        if ! sh_plugin_files.is_empty() {
            return Self::new_from_files(name, sh_plugin_files);
        }

        Self::new_from_files(name, vec![])
    }
}

fn main() {
    let default_zr_home = format!("{}/.zr", env!("HOME"));
    let zr_init = Path::new(option_env!("ZR_HOME").unwrap_or_else(|| &default_zr_home)).join("init.zsh");

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
    };
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
        .map(|plugin_name| Plugin::new(zr_init, &plugin_name))
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
            plugins.push(Plugin::new_from_files(name, vec![PathBuf::from(&file)]));
        }
    } else if !plugin_exists {
        plugins.push(Plugin::new(zr_init, name));
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

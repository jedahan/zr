use std::fmt;
use std::path::{Path, PathBuf, Component};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, ErrorKind, Write, LineWriter};
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

    pub fn new(name: &str) -> Plugin {
        let path = Path::new(&name);

        let files: Vec<_> = path.read_dir().unwrap()
            .filter_map(std::result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        // antigen style
        if let Some(antigen_plugin_file) = files.iter().find(|file| file.file_name().unwrap().to_string_lossy() == format!("{}.plugin.zsh", &name)) {
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
    let default_home = format!("{}/.zr", env!("HOME"));
    let zr_home = PathBuf::from(option_env!("ZR_HOME").unwrap_or(default_home.as_str()));

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "zsh plugin manager")
        (@subcommand reset => (about: "delete init.zsh") )
        (@subcommand debug => (about: "print debug information") )
        (@subcommand load =>
            (about: "load plugin")
            (@arg plugin: +required "plugin/name")
            (@arg file: "optional/path/to/file.zsh")
        )
    );

    match zr.clone().get_matches().subcommand() {
        ("load", Some(load_matches)) => {
            let plugin = load_matches.value_of("plugin").unwrap();
            let file = load_matches.value_of("file").unwrap();
            load(zr_home, plugin, file)
        },
        ("reset", _) => reset(zr_home),
        ("debug", _) => debug(zr_home),
        (_, _) => zr.print_help().unwrap()
    }
}

fn debug(zr_home: PathBuf) {
    println!("  ZR_HOME: {}", zr_home.display());
}

fn reset(zr_home: PathBuf) {
    if let Err(error) = fs::remove_file(zr_home.join("init.zsh")) {
        if error.kind() != ErrorKind::NotFound {
            Err(error).unwrap()
        }
    }
}

fn get_plugins_from(zr_home: PathBuf, filepath: &Path) -> Vec<Plugin> {
    let init_file = OpenOptions::new().read(true).open(&filepath).unwrap();

    BufReader::new(&init_file)
        .lines()
        .map(|line| line.unwrap())
        .filter(|line| line.starts_with("#"))
        .map(|line| line.split_whitespace().last().unwrap())
        .map(|plugin_name| Plugin::new(plugin_name))
        .collect::<Vec<Plugin>>()
}

fn load(zr_home: PathBuf, name: &str, file: &str) {
    let init_filename = format!("{}/init.zsh", zr_home.display());
    let mut plugins = get_plugins_from(zr_home, Path::new(&init_filename));

    if file != "" {
        let filepath = PathBuf::from(file);
        if let Some(plugin) = plugins.iter().find(|plugin| plugin.name == name) {
            plugin.files.insert(filepath);
        } else {
            plugins.push(Plugin::new_from_files(&name, vec![filepath]));
        }
    } else {
        plugins.push(Plugin::new(&name))
    }

    let temp_filename = format!("{}/init.zsh", std::env::temp_dir().display());
    let mut temp_file = OpenOptions::new().create_new(true).open(&temp_filename).unwrap();

    for plugin in plugins {
        writeln!(temp_file, "{}", plugin);
    }
    writeln!(temp_file, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump");

    fs::rename(&temp_filename, &init_filename).unwrap();
}

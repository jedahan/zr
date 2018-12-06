use clap::{clap_app, crate_version};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub mod error;
pub mod plugin;
pub mod plugins;

use crate::error::Error;
use crate::plugins::Plugins;

fn main() -> Result<(), Error> {
    let zr_home = get_var("ZR_HOME")?;
    let home = get_var("HOME")?;
    let default_home = format!("{}/.zr", home.unwrap());
    let path = PathBuf::from(zr_home.unwrap_or(default_home));

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "z:rat: - zsh plugin manager")
        (@subcommand list => (about: "list plugins") )
        (@subcommand load => (about: "load plugins fresh")
            (@arg plugins: +required +multiple +takes_value "plugin/name[/path/to/file.zsh] [[plugin/name [..]..]")
        )
        (@subcommand update => (about: "update plugins") )
    );

    match zr.clone().get_matches().subcommand() {
        ("list", _) => plugins_from(&path).list(),
        ("load", Some(m)) => load_plugins(&path, &m.values_of_lossy("plugins").unwrap()),
        ("update", _) => plugins_from(&path).update(),
        (_, _) => zr.print_help().map_err(Error::Clap),
    }
}

fn get_var(key: &str) -> Result<Option<String>, Error> {
    use std::env::VarError::{NotPresent, NotUnicode};

    match std::env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(NotPresent) => Ok(None),
        Err(NotUnicode(value)) => Err(Error::EnvironmentVariableNotUnicode {
            key: key.to_string(),
            value,
        }),
    }
}

pub fn plugins_from(zr_home: &PathBuf) -> Plugins {
    let mut plugins = Plugins::new(zr_home);
    let zr_init = &zr_home.join("init.zsh");
    let plugin_home = &zr_home.join("plugins");

    if zr_init.exists() {
        let init_file = OpenOptions::new().read(true).open(&zr_init).unwrap();
        for filepath in BufReader::new(&init_file)
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("source"))
            .map(|line| PathBuf::from(line.split_whitespace().last().unwrap()))
            .map(|filepath| filepath.strip_prefix(&plugin_home).ok().unwrap().to_owned())
            .collect::<Vec<_>>()
        {
            let _ = plugins.add(filepath.to_str().to_owned().unwrap());
        }
    }

    plugins
}

pub fn load_plugins(zr_home: &PathBuf, parameters: &[String]) -> Result<(), Error> {
    let mut plugins: Plugins = Plugins::new(zr_home);

    for param in parameters {
        let _ = plugins.add(param);
    }

    plugins.save()
}

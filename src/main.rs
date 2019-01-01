use clap::{clap_app, crate_version};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub mod error;
pub mod identifier;
pub mod plugin;
pub mod plugins;

use crate::error::Error;
use crate::identifier::Identifier;
use crate::plugins::Plugins;

fn zr_home() -> Result<String, Error> {
    let zr_home = get_var("ZR_HOME")?;
    let home = get_var("HOME")?;
    let default_home = format!("{}/.zr", home.unwrap());
    Ok(zr_home.unwrap_or(default_home))
}

fn main() -> Result<(), Error> {
    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "z:rat: - zsh plugin manager")
        (@arg home: -h --home +takes_value "Sets a custom directory for plugins")
        (@subcommand list => (about: "list plugins") )
        (@subcommand load => (about: "generate init.zsh from list of [http://example.com/]plugin/name[.git/path/to/file.zsh]")
            (@arg plugins: +required +multiple +takes_value "[http://example.com/]plugin/name[.git/path/to/file.zsh]")
        )
        (@subcommand update => (about: "update plugins") )
    );

    let matches = zr.clone().get_matches();
    let path = PathBuf::from(
        matches
            .value_of("home")
            .map(String::from)
            .unwrap_or_else(|| zr_home().unwrap()),
    );

    match matches.subcommand() {
        ("list", _) => plugins_from(&path).list(),
        ("update", _) => plugins_from(&path).update(),
        ("load", Some(m)) => load_plugins(&path, &m.values_of_lossy("plugins").unwrap()),
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

    if let Ok(init_file) = OpenOptions::new().read(true).open(&zr_init) {
        let _ = BufReader::new(&init_file)
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("# "))
            .map(|line| String::from(line.split_whitespace().last().unwrap()))
            .map(|uri| Identifier::from(uri))
            .try_for_each(|id| plugins.add(id));
    }

    plugins
}

pub fn load_plugins(zr_home: &PathBuf, parameters: &[String]) -> Result<(), Error> {
    let mut plugins: Plugins = Plugins::new(zr_home);

    for param in parameters {
        plugins.add(Identifier::from(param.to_string()))?;
    }

    plugins.save()
}

#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate libc;

use std::path::PathBuf;
use std::io::{BufRead, BufReader, Write};
use std::fs::OpenOptions;

mod error;
mod plugin;
mod plugins;

use plugins::*;
use error::*;

fn main() {
    if let Err(err) = run() {
        writeln!(&mut std::io::stderr(), "{}", err)
            .expect("error writing to stderr");
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

fn load_plugins_from(zr_home: &PathBuf) -> Plugins {
    let mut plugins = Plugins::new(zr_home.clone());
    let zr_init = &zr_home.join("init.zsh");
    let plugin_home = &zr_home.join("plugins");

    if zr_init.exists() {
        let init_file = OpenOptions::new().read(true).open(&zr_init).unwrap();
        for filepath in BufReader::new(&init_file)
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("source"))
            .map(|line| PathBuf::from(line.split_whitespace().last().unwrap()))
            .map(|filepath| filepath.strip_prefix(&plugin_home).ok().unwrap().to_owned() )
            .collect::<Vec<_>>() {
                let filename = filepath.to_str().to_owned().unwrap();
                let name = filename.split('/').collect::<Vec<_>>()[0..2].join("/");
                let file = filename.split('/').collect::<Vec<_>>()[2..].join("/");
                let _ = plugins.add(&name, Some(&file));
            }
    }

    plugins
}

fn run() -> Result<(), Error> {
    let zr_home = get_var("ZR_HOME")?;
    let home = get_var("HOME")?;
    let default_home = format!("{}/.zr", home.unwrap());
    let path = PathBuf::from(zr_home.unwrap_or(default_home));

    let mut plugins = load_plugins_from(&path);

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "z:rat: - zsh plugin manager")
        (@subcommand reset => (about: "delete init file") )
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
        (_, _) => zr.print_help().map_err(Error::Clap),
    }
}

//! # zr - a fast, friendly zsh package manager
//!
//! At its core, zr:
//!   * takes a list of urls to git repositories
//!   * downloads the code from those repos
//!   * and generates an init.zsh to setup paths and load zsh scripts for your zshrc
//!
use directories::ProjectDirs;
use std::env;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};

pub mod identifier;
pub mod plugin;
pub mod plugins;

use crate::identifier::Identifier;
use crate::plugins::Plugins;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Some(subcommand) = env::args().nth(1) {
        if subcommand.as_str() == "--update" {
            update();
            return;
        }
        if subcommand.as_str().contains('/') {
            let identifiers = env::args().skip(1).map(Identifier::new).collect();
            let plugins = load(identifiers);
            println!("{}", plugins);
            return;
        }
    }
    help()
}

fn help() {
    println!("
  {name} {version}
  by Jonathan Dahan <hi@jonathan.is>

  {name} [[http://example.com]plugin/name[.git/path/to/file.zsh]]    fetch plugins and output sourceable zsh
  {name} --update                                                    update plugins from already sourced zsh
  {name} help     show help", version=VERSION, name=NAME);
}

/// Take a list of identifiers (from cli args) and output sourceable zsh
pub fn load(identifiers: Vec<Identifier>) -> plugins::Plugins {
    let dirs = ProjectDirs::from("", "", NAME).expect("could not get cache directory");

    return Plugins::new(dirs.cache_dir(), identifiers);
}

pub fn update() {
    let _zr = env::var_os("_ZR").expect("_ZR env variable unset, bailing on update");

    if let Ok(init_file) = OpenOptions::new().read(true).open(&_zr) {
        let identifiers = BufReader::new(&init_file)
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("# "))
            .map(|line| String::from(line.split_whitespace().last().unwrap()))
            .map(|identifier| Identifier::new(identifier))
            .collect();

        let plugins = load(identifiers);
        plugins.update().expect("could not update plugins")
    }
}

#[test]
fn test_load() {
    let _input = "geometry-zsh/geometry.git/geometry.zsh zsh-users/zsh-autosuggestions";

    let _expected_output = "
export _ZR=$0
# geometry-zsh/geometry.git/geometry.zsh
source geometry.zsh

# zsh-users/zsh-autosuggestions
source /home/micro/.cache/zr/zsh-users/zsh-autosuggestions/zsh-autosuggestions.plugin.zsh
source /home/micro/.cache/zr/zsh-users/zsh-autosuggestions/zsh-autosuggestions.zsh
fpath+=/home/micro/.cache/zr/zsh-users/zsh-autosuggestions/
PATH=/home/micro/.cache/zr/zsh-users/zsh-autosuggestions:$PATH
";
}

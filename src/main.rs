//! # zr - a fast, friendly zsh package manager
//!
//! At its core, zr:
//!   * takes a list of urls to git repositories
//!   * downloads the code from those repos
//!   * and generates an init.zsh to setup paths and load zsh scripts for your zshrc
//!
extern crate directories;
extern crate git2_credentials;
extern crate url;

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
        if ["--update", "+update", "-u", "+u" ].contains(&subcommand.as_str()) {
            update();
            return;
        }
        if ["--help", "+help", "-h", "+h" ].contains(&subcommand.as_str()) {
            help();
            return;
        }
        if ["--version", "+version", "-v", "+v" ].contains(&subcommand.as_str()) {
            version();
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

fn version() {
    println!("{name} {version}", version=VERSION, name=NAME);
}

fn help() {
    version();
    println!("by Jonathan Dahan <hi@jonathan.is>

Example

. <(zr geometry-zsh/geometry junegunn/fzf.git/shell/key-bindings.zsh)

Format

{name} author/name                                    *.zsh from github.com/author/name
{name} author/name/file.zsh                        file.zsh from github.com/author/name
{name} https://gitlab.com/a/plugin                    *.zsh from gitlab.com/a/plugin
{name} https://gitlab.com/a/plugin.git/file.zsh    file.zsh from gitlab.com/a/plugin

Commands

{name} +update                                      update plugins from already sourced zsh
{name} +help                                        show help", name=NAME);
}

/// Take a list of identifiers (from cli args) and output sourceable zsh
pub fn load(identifiers: Vec<Identifier>) -> plugins::Plugins {
    let dirs = ProjectDirs::from("", "", NAME).expect("could not get cache directory");

    return Plugins::new(dirs.cache_dir(), identifiers);
}

pub fn update() {
    let zr = env::var_os("_ZR").expect("_ZR env variable unset, bailing on update");

    if let Ok(init_file) = OpenOptions::new().read(true).open(zr) {
        let identifiers = BufReader::new(&init_file)
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("# "))
            .map(|line| String::from(line.split_whitespace().last().unwrap()))
            .map(Identifier::new)
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

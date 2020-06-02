//! # zr - a fast, friendly zsh package manager
//!
//! At its core, zr:
//!   * takes a list of urls to git repositories
//!   * downloads the code from those repos
//!   * and generates an init.zsh to setup paths and load zsh scripts for your zshrc
//!
use directories::ProjectDirs;
use std::env;

pub mod identifier;
pub mod plugin;
pub mod plugins;

use crate::identifier::Identifier;
use crate::plugins::Plugins;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Some(subcommand) = env::args().nth(1) {
        if subcommand.as_str().contains('/') {
            load();
            return;
        }
    }
    help()
}

fn help() {
    println!("
  {name} {version}
  by Jonathan Dahan <hi@jonathan.is>

  {name} [[http://example.com]plugin/name[.git/path/to/file.zsh]]    fetch or update plugins and output sourceable zsh
  {name} help     show help", version=VERSION, name=NAME);
}

/// Take a list of identifiers (from cli args) and output sourceable zsh
pub fn load() {
    if let Some(dirs) = ProjectDirs::from("", "", NAME) {
        let cache = dirs.cache_dir();

        let identifiers = env::args().skip(1).map(Identifier::new).collect();

        let plugins = Plugins::new(cache, identifiers);

        println!("{}", plugins)
    }
}

#[test]
fn test_load() {
    let _input = "geometry-zsh/geometry.git/geometry.zsh zsh-users/zsh-autosuggestions";

    let _expected_output = "
# geometry-zsh/geometry.git/geometry.zsh
source geometry.zsh

# zsh-users/zsh-autosuggestions
source /home/micro/.cache/zr/zsh-users/zsh-autosuggestions/zsh-autosuggestions.plugin.zsh
source /home/micro/.cache/zr/zsh-users/zsh-autosuggestions/zsh-autosuggestions.zsh
fpath+=/home/micro/.cache/zr/zsh-users/zsh-autosuggestions/
PATH=/home/micro/.cache/zr/zsh-users/zsh-autosuggestions:$PATH

autoload -Uz compinit; compinit -iCd $HOME/.zcompdump
";
}

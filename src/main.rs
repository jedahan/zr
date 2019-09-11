//! # zr - a fast, friendly zsh package manager
//!
//! At its core, zr:
//!   * takes a list of urls to git repositories
//!   * downloads the code from those repos
//!   * and generates an init.zsh to setup paths and load zsh scripts for your zshrc
//!
use std::env;
use std::io::{self, Read, Result};
use std::path::PathBuf;

pub mod identifier;
pub mod plugin;
pub mod plugins;

use crate::identifier::Identifier;
use crate::plugins::Plugins;

fn cache() -> PathBuf {
    fn default_cache_home(_: env::VarError) -> String {
        format!("{}/.cache", env::var("HOME").unwrap())
    }

    PathBuf::from(env::var("XDG_CACHE_HOME").unwrap_or_else(default_cache_home)).join("zr")
}

/// We have three main commands
///
/// `load`: download and print sourceable zsh to load scripts
///
/// `update`: git pull all repositories found in the cache
///
/// `list`: list plugins in the cache
///
fn main() -> Result<()> {
    let path = cache();

    if let Some(subcommand) = env::args().nth(1) {
        return match subcommand.as_str() {
            "list" => plugins_from(&path).list(),
            "update" => plugins_from(&path).update(),
            "load" => load_plugins(&path, env::args().skip(2).collect()),
            _ => Ok(print_help()),
        };
    }
    Ok(())
}

fn print_help() {
    println!("
  zr {version}
  by Jonathan Dahan <hi@jonathan.is>

  zr help     show help
  zr list     list cached plugins
  zr update   update plugin repositories
  zr load     generate file to source from  [http://example.com/]plugin/name[.git/path/to/file.zsh]", version="0.9.0")
}

/// Create plugins from an existing `load` output
///
/// When we print plugins, the original identifier is stored as # { identifier }
///
pub fn plugins_from(config: &PathBuf) -> Plugins {
    let mut plugins = Plugins::new(config);

    let mut buffer = String::new();
    if let Ok(_) = io::stdin().read_to_string(&mut buffer) {
        let _ = buffer
            .lines()
            .filter(|line| line.starts_with("# "))
            .map(|line| String::from(line.split_whitespace().last().unwrap()))
            .map(|uri| Identifier::from(uri))
            .try_for_each(|id| plugins.add(id));
    }

    plugins
}

/// Take a list of identifiers (from cli args) and output sourceable zsh
pub fn load_plugins(cache: &PathBuf, parameters: Vec<String>) -> Result<()> {
    let mut plugins: Plugins = Plugins::new(cache);

    for param in parameters {
        plugins.add(Identifier::from(param.to_string()))?;
    }

    plugins.save()
}

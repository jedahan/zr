extern crate git2;

use std::collections::HashSet;
use std::ffi::OsStr;
use std::{fs, result};
use std::path::{Path, PathBuf};
use std::iter::FromIterator;
use std::cmp::{PartialEq, Eq};

use error::*;

#[derive(Clone,Debug)]
pub struct Plugin {
    pub host: String,
    pub author: String,
    pub name: String,
    pub files: HashSet<PathBuf>
}

impl PartialEq for Plugin {
    fn eq(&self, other: &Plugin) -> bool {
        self.host == other.host &&
        self.author == other.author &&
        self.name == other.name
    }
}
impl Eq for Plugin {}

/// A Plugin is an in-memory representation of
/// the author, name, and files to load
impl Plugin {
    pub fn add(&mut self, zr_home: PathBuf, file: PathBuf) {
        let full_path = zr_home
            .join("plugins")
            .join(&self.host)
            .join(&self.author)
            .join(&self.name)
            .join(&file);
        self.files.insert(full_path);
    }

    fn init(plugin_home: &Path, plugin_path: &Path) -> Result<(), Error> {
        let path = plugin_home.join(plugin_path);

        if ! path.is_dir() {
            fs::create_dir_all(&path).map_err(Error::Io)?;
        }

        if let Err(_) = git2::Repository::open(&path) {
            let url = format!("https://{}", &plugin_path.display());
            println!("cloning {} to {}", &url, &path.display());
            git2::Repository::clone(&url, path).map_err(Error::Git)?;
        }

        Ok(())
    }

    pub fn new(zr_home: &Path, host: &str, author: &str, name: &str) -> Result<Plugin, Error> {
        let plugin_home = zr_home.join("plugins");

        let id = [host, author, name].join("/");
        let plugin_path = PathBuf::from(&id);

        if plugin_path.components().count() > 3 {
            return Err(Error::InvalidIdentifier { id: id })
        }

        Plugin::init(&plugin_home, &plugin_path)?;

        let path = plugin_home.join(&plugin_path);

        let files: Vec<PathBuf> = path.read_dir().unwrap()
            .filter_map(result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        let sources: Vec<PathBuf> = {
            if let Some(antigen_plugin_file) = files.iter().find(|&file| *file == path.join(&name).with_extension("plugin.zsh")) {
                vec![antigen_plugin_file.to_owned()]
            } else if let Some(prezto_plugin_file) = files.iter().find(|&file| *file == path.join("init.zsh")) {
                vec![prezto_plugin_file.to_owned()]
            } else {
                let zsh_plugin_files: Vec<_> = files.iter().cloned().filter(|file| file.extension() == Some(OsStr::new("zsh"))).collect();
                if zsh_plugin_files.is_empty() {
                    files.iter().cloned().filter(|file| file.extension().unwrap() == "sh").collect()
                } else {
                    zsh_plugin_files
                }
            }
        };

        Ok(Plugin { host: host.to_string(), author: author.to_string(), name: name.to_string(), files: HashSet::from_iter(sources) } )
    }
}

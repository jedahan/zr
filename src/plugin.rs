use std::collections::HashSet;
use std::ffi::OsStr;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{fmt, fs, result};

use crate::error::Error;

crate struct Plugin {
    pub author: String,
    pub name: String,
    pub files: HashSet<PathBuf>,
}

/// A Plugin is an in-memory representation of
/// the author, name, and files to load
impl Plugin {
    fn clone_if_empty(path: &Path, author: &str, name: &str) -> Result<(), Error> {
        if !path.is_dir() {
            let parent = path.parent().unwrap();
            if !parent.exists() {
                fs::create_dir(parent).map_err(Error::Io)?;
            }

            let url = format!("https://github.com/{}/{}", author, name);
            println!("cloning {}", url);
            git2::Repository::clone(&url, &path).unwrap();
        }
        Ok(())
    }

    pub fn new(zr_home: &Path, author: &str, name: &str) -> Result<Plugin, Error> {
        let plugin_home = zr_home.join("plugins");
        if !plugin_home.exists() {
            fs::create_dir_all(&plugin_home)
                .expect(format!("error creating plugin dir '{:?}'", &plugin_home).as_str());
        }
        let path = zr_home.join("plugins").join(&author).join(&name);

        Plugin::clone_if_empty(&path, author, name)?;

        let files: Vec<PathBuf> = path
            .read_dir()
            .unwrap()
            .filter_map(result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        let sources: Vec<PathBuf> = {
            if let Some(antigen_plugin_file) = files
                .iter()
                .find(|&file| *file == path.join(&name).with_extension("plugin.zsh"))
            {
                vec![antigen_plugin_file.to_owned()]
            } else if let Some(prezto_plugin_file) =
                files.iter().find(|&file| *file == path.join("init.zsh"))
            {
                vec![prezto_plugin_file.to_owned()]
            } else {
                let zsh_plugin_files: Vec<_> = files
                    .iter()
                    .cloned()
                    .filter(|file| file.extension() == Some(OsStr::new("zsh")))
                    .collect();
                if zsh_plugin_files.is_empty() {
                    files
                        .iter()
                        .cloned()
                        .filter(|file| file.extension().unwrap() == "sh")
                        .collect()
                } else {
                    zsh_plugin_files
                }
            }
        };

        Ok(Plugin {
            author: author.to_string(),
            name: name.to_string(),
            files: HashSet::from_iter(sources),
        })
    }

    pub fn from_files(zr_home: &Path, author: &str, name: &str, files: Vec<PathBuf>) -> Plugin {
        let path = zr_home.join("plugins").join(&author).join(&name);
        let _ = Plugin::clone_if_empty(&path, author, name);

        let mapped = files.iter().cloned().map(|file| path.join(&file)).collect();

        Plugin {
            author: author.to_string(),
            name: name.to_string(),
            files: mapped,
        }
    }
}

impl fmt::Display for Plugin {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut basedirs = HashSet::new();
        writeln!(formatter, "# {}/{}", self.author, self.name)?;
        for file in &self.files {
            if let Some(basedir) = file.parent() {
                basedirs.insert(basedir);
            }
            if let Some(filename) = file.to_str() {
                writeln!(formatter, "source {}", filename.replace("\\", "/"))?;
            }
        }

        for basedir in basedirs.iter().filter_map(|b| b.to_str()) {
            let dir = basedir.replace("\\", "/");
            writeln!(formatter, "fpath+={}/", dir)?;
            writeln!(formatter, "PATH={}:$PATH", dir)?;
        }

        Ok(())
    }
}

use std::collections::HashSet;
use std::ffi::OsStr;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{fmt, fs, result};

use crate::error::Error;
use crate::identifier::Identifier;

pub struct Plugin {
    pub identifier: Identifier,
    pub files: HashSet<PathBuf>,
}

/// A Plugin is an in-memory representation of
/// the author, name, and files to load
impl Plugin {
    fn clone_if_empty(source: &str, path: &Path) -> Result<(), Error> {
        if !path.is_dir() {
            println!("cloning {} into {:?}", source, path);
            git2::Repository::clone(&source, &path).unwrap();
        }
        Ok(())
    }

    pub fn new(zr_home: &Path, identifier: Identifier) -> Result<Plugin, Error> {
        let plugin_home = zr_home.join("plugins");
        if !plugin_home.exists() {
            fs::create_dir_all(&plugin_home)
                .unwrap_or_else(|_| panic!("error creating plugin dir '{:?}'", &plugin_home));
        }
        let repository = identifier.repository();
        let name = identifier.name();
        let path = zr_home.join(&name);

        Plugin::clone_if_empty(&repository, &path)?;

        if let Ok(filepath) = identifier.filepath() {
            if filepath.iter().count() > 0 {
                return Ok(Plugin {
                    identifier,
                    files: [path.join(filepath)].iter().cloned().collect(),
                })
            }
        };

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
            identifier,
            files: HashSet::from_iter(sources),
        })
    }

    pub fn from_file(zr_home: &Path, identifier: Identifier, file: PathBuf) -> Plugin {
        let path = zr_home.join(identifier.name());
        let _ = Plugin::clone_if_empty(&identifier.source(), &path);
        let mut files = HashSet::new();
        files.insert(path.join(file));

        Plugin { identifier, files }
    }
}

impl fmt::Display for Plugin {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut basedirs = HashSet::new();
        writeln!(formatter, "# {}", self.identifier.source())?;
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

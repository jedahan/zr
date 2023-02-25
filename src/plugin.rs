use std::collections::HashSet;
use std::ffi::OsStr;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{fmt, result};

use crate::identifier::Identifier;
//#[cfg(!windows)]
use std::os::unix::fs::PermissionsExt;

/// A Plugin is an in-memory representation of the identifier and files to load
pub struct Plugin {
    pub identifier: Identifier,
    pub files: HashSet<PathBuf>,
}

impl Plugin {
    /// Simple git clone; does not support ssh authentication yet
    fn clone_if_empty(source: &url::Url, path: &Path) -> Result<(), std::io::Error> {
        if !path.is_dir() {
            eprintln!("cloning {} into {:?}", source, path);
            git2::Repository::clone(source.as_str(), path).unwrap();
        }
        Ok(())
    }

    /// The only thing you need to know is an identifier and the cache directory
    ///
    /// Side-effects include
    ///
    /// * attempting to create the cache if it does not exist
    /// * downloading the repo if it is empty
    ///
    pub fn new(cache: &Path, identifier: Identifier) -> Result<Plugin, std::io::Error> {
        let Identifier {
            ref url,
            ref dir,
            ref file,
            ..
        } = identifier.clone();
        let path = &cache.join(dir);
        Plugin::clone_if_empty(url, path)?;

        // If we were given an Identifier with a file, return a plugin with just that file
        if let Some(file) = file {
            let mut files = HashSet::with_capacity(1);
            files.insert(path.join(file));
            return Ok(Plugin { identifier, files });
        };

        // Get a list of all files with an extension
        let files: Vec<PathBuf> = path
            .read_dir()
            .unwrap()
            .filter_map(result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file())
            .collect();

        let name = dir.components().last().unwrap();

        let sources: Vec<PathBuf> = {
            if let Some(antigen_plugin_file) = files
                .iter()
                .find(|&file| *file == path.join(name).with_extension("plugin.zsh"))
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
                if !zsh_plugin_files.is_empty() {
                    zsh_plugin_files
                } else {
                    let completion_files: Vec<_> = files
                        .iter()
                        .cloned()
                        .filter(|file| {
                            if let Some(name) = file.file_name() {
                                name.to_string_lossy().starts_with('_')
                            } else {
                                false
                            }
                        })
                        .collect();
                    if !completion_files.is_empty() {
                        completion_files
                    } else {
                        files
                            .iter()
                            .cloned()
                            .filter(|file| file.extension() == Some(OsStr::new("sh")))
                            .collect()
                    }
                }
            }
        };

        Ok(Plugin {
            identifier,
            files: HashSet::from_iter(sources),
        })
    }
}

/// This actually is the serialization for init.zsh
impl fmt::Display for Plugin {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut basedirs = HashSet::new();
        writeln!(formatter, "# {}", self.identifier.name)?;
        for file in &self.files {
            if let Some(basedir) = file.parent() {
                basedirs.insert(basedir);
            }
            if let Some(filename) = file.to_str() {
                if !file.file_name().unwrap().to_string_lossy().starts_with('_') {
                    writeln!(formatter, "source {}", filename.replace('\\', "/"))?;
                }
            }
        }

        if cfg!(windows) {
            for basedir in basedirs.iter() {
                let dir = basedir.to_string_lossy().replace('\\', "/");
                writeln!(formatter, "fpath+={}/", dir)?;
                writeln!(formatter, "PATH={}:$PATH", dir)?;
            }
        } else {
            // Add directories to fpath and PATH if we find any executable file
            for dir in basedirs.iter() {
                if let Ok(files) = dir.read_dir() {
                    let has_exe = files
                        .filter_map(|files| files.ok())
                        .filter_map(|direntry| direntry.metadata().ok())
                        .filter(|metadata| metadata.is_file())
                        .any(|metadata| metadata.permissions().mode() & 0o111 != 0);
                    let has_completions = dir
                        .read_dir()
                        .unwrap()
                        .filter_map(|files| files.ok())
                        .any(|direntry| direntry.file_name().to_string_lossy().starts_with('_'));
                    let has_functions = dir
                        .read_dir()
                        .unwrap()
                        .filter_map(|files| files.ok())
                        .any(|direntry| direntry.file_name().to_string_lossy() == "functions");
                    if has_exe {
                        writeln!(formatter, "PATH={}:$PATH", dir.display())?;
                    }
                    if has_exe || has_completions {
                        writeln!(formatter, "fpath+={}/", dir.display())?;
                    }
                    if has_functions {
                        writeln!(formatter, "fpath+={}/functions", dir.display())?;
                    }
                }
            }
        }

        Ok(())
    }
}

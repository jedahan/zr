use std::fmt;
use std::fs::create_dir_all;
use std::io::Error;
use std::path::{Path, PathBuf};

use git2_credentials::CredentialHandler;

use crate::identifier::Identifier;
use crate::plugin::Plugin;

/// Plugins are collected into different `home`s
pub struct Plugins {
    plugins: Vec<Plugin>,
    cache: PathBuf,
}

impl Plugins {
    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let dir = self.cache.join(&plugin.identifier.dir);
            let repo = git2::Repository::open(&dir).unwrap();
            let mut callbacks = git2::RemoteCallbacks::new();

            let git_config = git2::Config::open_default().unwrap();
            let mut credential_handler = CredentialHandler::new(git_config);
            callbacks.credentials(move |url, username, allowed| {
                credential_handler.try_next_credential(url, username, allowed)
            });

            callbacks.update_tips(|refspec, from, to| {
                eprintln!(
                    "{} {} {:.6} ↓ {:.6}",
                    &plugin.identifier.name,
                    refspec.replace("refs/heads/", ""),
                    from,
                    to
                );
                true
            });
            let mut options = git2::FetchOptions::new();
            options.remote_callbacks(callbacks);

            let mut remote = repo.find_remote("origin").unwrap();
            remote
                .fetch(&["refs/heads/*:refs/heads/*"], Some(&mut options), None)
                .unwrap();
        }
        Ok(())
    }

    pub fn new(cache: &Path, identifiers: Vec<Identifier>) -> Plugins {
        if !cache.exists() {
            create_dir_all(&cache).expect("failed to create the cache directory");
        }

        let mut plugins: Vec<Plugin> = vec![];

        for identifier in identifiers {
            if let Some(plugin) = plugins
                .iter_mut()
                .find(|plugin| plugin.identifier == identifier)
            {
                if let Some(file) = &identifier.file {
                    plugin.files.insert(cache.join(&identifier.dir).join(&file));
                }
            };

            if let Ok(plugin) = Plugin::new(cache, identifier) {
                plugins.push(plugin);
            }
        }

        Plugins {
            plugins,
            cache: cache.to_path_buf(),
        }
    }
}

impl fmt::Display for Plugins {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "export _ZR=$0")?;
        for plugin in &self.plugins {
            writeln!(formatter, "{}", plugin)?;
        }
        Ok(())
    }
}

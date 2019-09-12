use std::fmt;
use std::fs::create_dir_all;
use std::io::Error;
use std::path::PathBuf;

use git2_credentials::CredentialHandler;

use crate::identifier::Identifier;
use crate::plugin::Plugin;

/// Plugins are collected into different `home`s
pub struct Plugins {
    cache: PathBuf,
    plugins: Vec<Plugin>,
}

impl Plugins {
    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let plugin_home = self.cache.join(&plugin.identifier.name());
            let repo = git2::Repository::open(&plugin_home).unwrap();
            let mut remote = repo.find_remote("origin").unwrap();
            let mut callbacks = git2::RemoteCallbacks::new();

            let git_config = git2::Config::open_default().unwrap();
            let mut ch = CredentialHandler::new(git_config);
            callbacks.credentials(move |url, username, allowed| {
                ch.try_next_credential(url, username, allowed)
            });

            callbacks.update_tips(|refspec, from, to| {
                println!(
                    "updated {} {} from {:.6}..{:.6}",
                    refspec, &plugin.identifier, from, to
                );
                true
            });
            let mut options = git2::FetchOptions::new();
            options.remote_callbacks(callbacks);
            // TODO: remove hardcoding of master
            remote
                .fetch(
                    &["refs/heads/master:refs/heads/master"],
                    Some(&mut options),
                    None,
                )
                .unwrap();
        }
        Ok(())
    }

    pub fn new(cache: &PathBuf) -> Plugins {
        if !cache.exists() {
            create_dir_all(&cache).expect("failed to create the cache directory");
        }
        Plugins {
            cache: cache.clone(),
            plugins: vec![],
        }
    }

    pub fn list(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            println!("{}", plugin.identifier)
        }
        Ok(())
    }

    // Checks to see if a plugin has already been loaded
    // If it has been loaded, add the current file to the plugin
    // Else add a new plugin from scratch
    pub fn add(&mut self, identifier: Identifier) -> Result<(), Error> {
        if let Some(plugin) = self
            .plugins
            .iter_mut()
            .find(|plugin| plugin.identifier == identifier)
        {
            if let Ok(filepath) = identifier.filepath() {
                if filepath.iter().count() > 0 {
                    plugin.files.insert(self.cache.join(filepath));
                }
            }
            return Ok(());
        };

        if let Ok(plugin) = Plugin::new(&self.cache, identifier) {
            self.plugins.push(plugin);
        }

        Ok(())
    }

    // Serialize all the plugins to $ZR_HOME/init.zsh
    pub fn save(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            println!("{}", plugin);
        }
        println!("autoload -Uz compinit; compinit -iCd $HOME/.zcompdump");

        Ok(())
    }
}

impl fmt::Display for Plugins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.cache.display())?;
        for plugin in &self.plugins {
            writeln!(f, "{}", plugin)?;
        }
        Ok(())
    }
}

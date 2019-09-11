use std::fs::OpenOptions;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::{env, fmt, fs};

use git2_credentials::CredentialHandler;

use crate::identifier::Identifier;
use crate::plugin::Plugin;

/// Plugins are collected into different `home`s
pub struct Plugins {
    home: PathBuf,
    plugins: Vec<Plugin>,
}

impl Plugins {
    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let plugin_home = self.home.join(&plugin.identifier.name());
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

    pub fn new(zr_home: &PathBuf) -> Plugins {
        if !zr_home.exists() {
            fs::create_dir_all(&zr_home)
                .unwrap_or_else(|_| panic!("error creating zr_home dir '{:?}'", &zr_home));
        }
        Plugins {
            home: zr_home.clone(),
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
            .find(|plugin| &plugin.identifier == &identifier)
        {
            if let Ok(filepath) = identifier.filepath() {
                if filepath.iter().count() > 0 {
                    plugin.files.insert(self.home.join(filepath));
                }
            }
            return Ok(());
        };

        if let Ok(plugin) = Plugin::new(&self.home, identifier) {
            self.plugins.push(plugin);
        }

        Ok(())
    }

    // Serialize all the plugins to $ZR_HOME/init.zsh
    pub fn save(&self) -> Result<(), Error> {
        let filename = "init.zsh";
        let temp_file_path = env::temp_dir().join(filename);
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_file_path)
            .expect("temp file");

        for plugin in &self.plugins {
            writeln!(temp_file, "{}", plugin).expect("Should be able to write to temp_file");
        }
        writeln!(
            temp_file,
            "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump"
        )
        .expect("Should be able to write the autoload line");

        let dst_file_path = &self.home.join(filename);
        fs::copy(&temp_file_path, &dst_file_path).expect("Should be able to copy to dst_file");
        fs::remove_file(&temp_file_path).expect("Should be able to remove temp_file");
        Ok(())
    }
}

impl fmt::Display for Plugins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.home.display())?;
        for plugin in &self.plugins {
            writeln!(f, "{}", plugin)?;
        }
        Ok(())
    }
}

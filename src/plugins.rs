use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fmt, fs};

use crate::error::Error;
use crate::plugin::Plugin;

pub struct Plugins {
    home: PathBuf,
    plugins: Vec<Plugin>,
}

impl Plugins {
    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let plugin_home = self
                .home
                .join("plugins")
                .join(&plugin.author)
                .join(&plugin.name);
            let repo = git2::Repository::open(&plugin_home).map_err(Error::Git)?;
            let mut remote = repo.find_remote("origin").map_err(Error::Git)?;
            let mut callbacks = git2::RemoteCallbacks::new();
            callbacks.update_tips(|refspec, from, to| {
                println!(
                    "updated {} {}/{} from {:.6}..{:.6}",
                    refspec, &plugin.author, &plugin.name, from, to
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
                .map_err(Error::Git)?;
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
            println!("{}/{}", plugin.author, plugin.name)
        }
        Ok(())
    }

    pub fn add(&mut self, filename: &str) -> Result<(), Error> {
        if filename.split('/').count() < 2 {
            return Err(Error::InvalidPluginName {
                plugin_name: filename.to_string(),
            });
        }

        let mut fileiter = filename.split('/');

        let author = fileiter.next().unwrap().to_string();
        let name = fileiter.next().unwrap().to_string();
        let file = fileiter.collect::<Vec<_>>().join("/");
        let filepath = self
            .home
            .join("plugins")
            .join(&author)
            .join(&name)
            .join(&file);

        if let Some(plugin) = self
            .plugins
            .iter_mut()
            .find(|plugin| (&plugin.name, &plugin.author) == (&name, &author))
        {
            if file != "" {
                plugin.files.insert(filepath);
            }
            return Ok(());
        };

        let plugin = if file != "" {
            Plugin::from_file(&self.home, &author, &name, filepath)
        } else {
            Plugin::new(&self.home, &author.to_string(), &name)?
        };

        self.plugins.push(plugin);
        Ok(())
    }

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

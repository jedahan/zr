extern crate git2;

use plugin::Plugin;
use error::Error;

use std::{env, fmt, fs};
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

pub struct Plugins {
    home: PathBuf,
    plugins: Vec<Plugin>
}

impl Plugins {
    pub fn reset(&self) -> Result<(), Error> {
        let filepath = self.home.join("init.zsh");
        fs::remove_file(&filepath).or_else(|error|
             if error.kind() == ErrorKind::NotFound {
                 Ok(())
             } else {
                 Err(Error::Io(error))
             })
    }

    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let plugin_home = self.home.join("plugins").join(&plugin.author).join(&plugin.name);
            if let Ok(repo) = git2::Repository::open(&plugin_home) {
                if let Ok(remotes) = repo.remotes() {
                    if let Some(first_remote) = remotes.get(0) {
                        let mut cb = git2::RemoteCallbacks::new();
                        cb.update_tips(|_, a, b| {
                            if ! a.is_zero() {
                                println!("updated {}/{} from {:6}..{:6}", &plugin.author, &plugin.name, a, b);
                            }
                            true
                        });
                        let mut opts = git2::FetchOptions::new();
                        opts.remote_callbacks(cb);
                        let mut remote = repo.find_remote(first_remote).unwrap();
                        let refspec = "refs/heads/*:refs/heads/*";
                        remote.fetch(&[refspec], Some(&mut opts), None).map_err(Error::Git)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn new(zr_home: PathBuf) -> Plugins {
        Plugins {
            home: zr_home.clone(),
            plugins: vec![]
        }
    }

    pub fn list(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            println!("{}/{}", plugin.author, plugin.name)
        }
        Ok(())
    }

    pub fn add(&mut self, plugin_name: &str, file: Option<&str>) -> Result<(), Error> {
        let plugin_path = PathBuf::from(plugin_name);
        if plugin_path.components().count() != 2 {
            return Err(Error::InvalidPluginName { plugin_name: plugin_name.to_string() })
        }

        let name = plugin_path.components().last().unwrap().as_os_str().to_str().unwrap().to_string();
        let author = plugin_path.parent().unwrap().components().last().unwrap().as_os_str().to_str().unwrap().to_string();

        if let Some(filepath) = file {
            if self.plugins.iter().find(|plugin| (&plugin.name, &plugin.author) == (&name, &author)).is_none() {
                let files = vec![PathBuf::from(&filepath)];
                let plugin = Plugin::from_files(&self.home, &author, &name, files);
                self.plugins.push(plugin);
            } else if let Some(plugin) = self.plugins.iter_mut().find(|plugin| (&plugin.name, &plugin.author) == (&name, &author)) {
                let file = self.home.join("plugins").join(&author).join(&name).join(&filepath);
                plugin.files.insert(file);
            }
        }

        if file.is_none() && self.plugins.iter().all(|plugin| (&plugin.name, &plugin.author) != (&name, &author)) {
            let plugin = Plugin::new(&self.home, &author, &name)?;
            self.plugins.push(plugin);
        }

        let filename = "init.zsh";
        let temp_file_path = env::temp_dir().join(filename);
        let mut temp_file = OpenOptions::new().write(true).create(true).truncate(true).open(&temp_file_path).expect("temp file");

        for plugin in &self.plugins {
            writeln!(temp_file, "{}", plugin)
                .expect("Should be able to write to temp_file");
        }
        writeln!(temp_file, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
            .expect("Should be able to write the autoload line");

        let dst_file_path = &self.home.join(filename);
        fs::copy(&temp_file_path, &dst_file_path).expect("Should be able to dst_file");
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

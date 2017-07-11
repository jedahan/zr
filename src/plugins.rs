extern crate git2;
extern crate url;

use plugin::Plugin;
use error::Error;

use std::{env, fmt, fs};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use self::url::Url;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Plugins {
    home: PathBuf,
    plugins: Vec<Plugin>
}

impl Plugins {
    pub fn update(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            let plugin_home = self.home.join("plugins").join(&plugin.author).join(&plugin.name);
            if let Ok(repo) = git2::Repository::open(&plugin_home) {
                if let Ok(remotes) = repo.remotes() {
                    if let Some(first_remote) = remotes.get(0) {
                        let mut cb = git2::RemoteCallbacks::new();
                        cb.update_tips(|_, a, b| {
                            if ! a.is_zero() {
                                println!("updated {}/{} from {:.6}..{:.6}", &plugin.author, &plugin.name, a, b);
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
        if ! zr_home.exists() {
            fs::create_dir_all(&zr_home)
                .expect(format!("error creating zr_home dir '{:?}'",&zr_home).as_str());
        }
        Plugins {
            home: zr_home.clone(),
            plugins: vec![]
        }
    }

    pub fn list(&self) -> Result<(), Error> {
        for plugin in &self.plugins {
            println!("{}/{}/{}", plugin.host, plugin.author, plugin.name)
        }
        Ok(())
    }

    pub fn resolve(id: &str) -> Result<Url, Error> {
        use self::url::ParseError::*;

        let options = Url::options();
        let github = Url::parse("https://github.com")
            .map_err(Error::Url)?;
        let github_url = options.base_url(Some(&github));

        let uri = match Url::parse(id) {
            Ok(value) => Ok(value),
            Err(RelativeUrlWithoutBase) => {
                github_url.parse(id).map_err(Error::Url)
            },
            Err(e) => Err(e).map_err(Error::Url)
        };

        let uri = uri?;

        if uri.cannot_be_a_base() {
            return Err(Error::InvalidIdentifier { id: id.to_string() })
        }

        Ok(uri)
    }

    pub fn add(&mut self, id: &str) -> Result<(), Error> {
        // this is failing with id = github.com/author/name
        let uri = Plugins::resolve(id)?;

        if uri.path_segments().unwrap().count() < 2 {
            return Err(Error::InvalidPluginName { plugin_name: id.to_string() })
        }

        let mut path_segments = uri.path_segments().unwrap();

        let host = uri.host().unwrap().to_string();
        let author = path_segments.next().unwrap().to_string();
        let name = path_segments.next().unwrap().to_string();

        let plugin_to_add = Plugin::new(&self.home, &host, &author, &name)?;
        let file = PathBuf::from(path_segments.collect::<Vec<_>>().join("/"));

        if ! self.plugins.contains(&plugin_to_add) {
            self.plugins.push(plugin_to_add.clone());
        }

        if let Some(plugin) = self.plugins.iter_mut().find(|p| *p == &plugin_to_add) {
            plugin.add(self.home.clone(), file);
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), Error> {
        let filename = "init.zsh";
        let temp_file_path = env::temp_dir().join(filename);
        let mut temp_file = OpenOptions::new().write(true).create(true).truncate(true).open(&temp_file_path).expect("temp file");

        writeln!(temp_file, "{}", &self)
            .expect("Should be able to write to temp_file");
        writeln!(temp_file, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
            .expect("Should be able to write the autoload line");

        let dst_file_path = &self.home.join(filename);
        fs::copy(&temp_file_path, &dst_file_path).expect("Should be able to copy to dst_file");
        fs::remove_file(&temp_file_path).expect("Should be able to remove temp_file");
        Ok(())
    }
}

impl fmt::Display for Plugins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for plugin in &self.plugins {
            let mut basedirs = HashSet::new();
            writeln!(f, "# {}/{}/{}", plugin.host, plugin.author, plugin.name)?;
            for file in &plugin.files {
                if let Some(basedir) = file.parent() {
                    basedirs.insert(basedir);
                }
                writeln!(f, "source {}", &self.home.join(file).display())?;
            }
            for basedir in basedirs {
                writeln!(f, "fpath+={}/", basedir.display())?;
                writeln!(f, "PATH={}:$PATH", basedir.display())?;
            }
        }
        Ok(())
    }
}

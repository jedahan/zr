extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::path::PathBuf;
// use serde::*;
// pub use ser::{to_string, Serializer};
// pub use de::{from_str, Deserializer};

const VERSION: &'static str = "0.0.1";

#[derive(Serialize, Deserialize, Debug)]
struct Plugin {
    repo: String,
    files: Vec<PathBuf>
}

impl Plugin {
    pub fn new(prefix: &PathBuf, repo_name: &PathBuf) -> Plugin {
        let name = repo_name.iter().last().unwrap();
        let path = prefix.join("plugins").join(repo_name.clone());
        let repo = String::from(repo_name.to_string_lossy());

        // antigen style plugins
        let antigen_plugin_file = path.join(name).join(".plugin.zsh");
        if antigen_plugin_file.exists() {
            return Plugin {
                repo: repo,
                files: vec![antigen_plugin_file],
            }
        }

        // prezto style plugins
        let prezto_plugin_file = path.join("init.zsh");
        if prezto_plugin_file.exists() {
            return match std::process::Command::new("pmodload").arg(name.clone()).spawn() {
                Ok(_) =>
                    Plugin {
                        repo: repo,
                        files: vec![],
                    },
                Err(_) =>
                    Plugin {
                        repo: repo,
                        files: vec![prezto_plugin_file],
                    }
            }
        }

        let mut filenames = path.read_dir().unwrap()
            .filter_map(std::result::Result::ok)
            .map(|entry| PathBuf::from(entry.file_name().to_string_lossy().into_owned()));

        // zsh plugins
        if filenames.any(|filename| filename.ends_with(".zsh")) {
            let zsh_files = filenames.filter(|filename| filename.ends_with(".zsh"));
            return Plugin {
                repo: repo,
                files: zsh_files.collect(),
            }
        }

        // sh plugins
        if filenames.any(|filename| filename.ends_with(".sh")) {
            let sh_files = filenames.filter(|filename| filename.ends_with(".zsh"));
            return Plugin {
                repo: repo,
                files: sh_files.collect(),
            }
        }

        Plugin { repo: repo, files: vec![] }
    }
}

// Plugin {
//   repo: "frmendes/prompt-geometry",
//   files: [ "geometry.zsh" ],
// }
//
// source /Users/jedahan/.zr/plugins/frmendes/prompt-geometry/geometry.zsh
// fpath+=/Users/jedahan/.zr/plugins/frmendes/prompt-geometry/
// PATH=/Users/jedahan/.zr/plugins/frmendes/prompt-geometry:$PATH

// impl ser::to_string<Zsh> for Plugin {
//    pub fn to_string<Zsh>(&self) -> str {
//        let f = io::Writer;
//        for file in self.files {
//            write!(f, "# {repo_name}
//                source {prefix}{file}
//                fpath+={prefix}{fpath}/
//                PATH={prefix}{fpath}:$PATH",
//                repo_name = self.repo,
//                prefix = format!("{}/{}", ZR_HOME, self.repo),
//                fpath = file.strip("/*$"),
//                ).unwrap();
//        }
//    }
// }

fn main() {
    let zr_home = PathBuf::from(option_env!("ZR_HOME").unwrap_or(format!("{}/.zr", env!("HOME")).as_str()));

    match env::args().nth(1) {
        Some(command) => {
            match command.as_ref() {
                "version" => version(),
                "debug" => debug(zr_home),
                "load" => load(zr_home),
                _ => help(),
            }
        },
        None => help()
    };
}

fn debug(zr_home: PathBuf) {
    println!("env:");
    println!("  ZR_HOME: {}", zr_home.to_string_lossy());
}

fn help() {
    println!("zr {}", VERSION);
    println!();
    println!("usage:");
    println!("  zr [<plugin>|command]");
    println!();
    println!("  zr plugin - save 'plugin' to ~/.zr-init.zsh")
    println!();
    println!("commands:");
    println!("  zr help - print this help");
    println!("  zr version - print the version");
}

fn version() {
    println!("{}", VERSION);
}

fn load(zr_home: PathBuf) {
    let repo_name = PathBuf::from(env::args().nth(2).unwrap());
    println!("loading {:?}", repo_name);
    let plugin = Plugin::new(&zr_home, &repo_name);
    println!("loaded {:?}", plugin);
}

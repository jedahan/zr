extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::path::PathBuf;
use serde::*;
// pub use ser::{to_string, Serializer};
// pub use de::{from_str, Deserializer};

const VERSION: &'static str = "0.0.1";

#[derive(Serialize, Deserialize, Debug)]
struct Plugin {
    repo: String,
    files: Vec<std::path::PathBuf>
}

impl Plugin {
    pub fn new(prefix: &std::path::PathBuf, name: String) -> Plugin {
        let plugin_path = prefix.join("plugins").join(name.clone());
        Plugin {
            repo: name.clone(),
            files: plugin_path.read_dir().unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(".zsh"))
                .map(|entry| entry.path())
                .collect(),
        }
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

fn debug(zr_home: std::path::PathBuf) {
    println!("env:");
    println!("  ZR_HOME: {}", zr_home.to_string_lossy());
}

fn help() {
    println!("usage:");
    println!("  zr help - print this help");
    println!("  zr version - print the version");
}

fn version() {
    println!("{}", VERSION);
}

fn load(zr_home: std::path::PathBuf) {
    let plugin_name = env::args().nth(2).unwrap();
    println!("loading {}", plugin_name);
    let plugin = Plugin::new(&zr_home, plugin_name);
    println!("loaded {:?}", plugin);
}

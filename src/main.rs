use std::{env,fmt};
use std::path::Path;

const VERSION: &'static str = "0.0.1";

struct Plugin<'a> {
    repo: &'a Path,
    files: Vec<&'a Path>
}

impl<'a> fmt::Display for Plugin<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = self.repo.display();
        for file in self.files {
            let basedir = file.parent().unwrap().display();
            write!(f, r"source {}/{}", prefix, file.display())?;
            write!(f, r"fpath+={}/{}/", prefix, basedir)?;
            write!(f, r"PATH=={}/{}:$PATH", prefix, basedir)?;
        }
        write!(f, "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump")
    }
}

impl<'a> Plugin<'a> {
    pub fn from_path(path: &'a Path) -> Plugin<'a> {
        let name = path.iter().last().unwrap();
        let mut files = path.read_dir().unwrap().filter_map(std::result::Result::ok).map(|file| file.path().as_path());

        // antigen style plugins
        if files.any(|file| *file == path.join(name).join(".plugin.zsh")) {
            let _files = files.filter(|file| *file == path.join(name).join(".plugin.zsh")).collect();
            return Plugin {
                repo: path,
                files: _files
            }
        }

        // prezto style plugins
        if files.any(|file| *file == *path.join("init.zsh")) {
            let _files = files.filter(|file| *file == path.join("init.zsh")).collect();
            return match std::process::Command::new("pmodload").arg(name.clone()).spawn() {
                Ok(_) =>
                    Plugin {
                        repo: path,
                        files: vec![]
                    },
                Err(_) =>
                    Plugin {
                        repo: path,
                        files: _files
                    }
            }
        }

        // zsh plugins
        if files.any(|file| file.ends_with(".zsh")) {
            return Plugin {
                repo: path,
                files: files.filter(|file| file.ends_with(".zsh")).collect(),
            }
        }

        // sh plugins
        if files.any(|file| file.ends_with(".sh")) {
            return Plugin {
                repo: path,
                files: files.filter(|file| file.ends_with(".sh")).collect(),
            }
        }

        Plugin { repo: path, files: vec![] }
    }
}

fn main() {
    let default_home = format!("{}/.zr", env!("HOME"));
    let zr_home = Path::new(option_env!("ZR_HOME").unwrap_or(default_home.as_str()));

    match env::args().nth(1) {
        Some(command) => {
            match command.as_ref() {
                "version" => version(),
                "debug" => debug(&zr_home),
                "load" => load(&zr_home, &Path::new(&env::args().nth(2).unwrap())),
                _ => help(),
            }
        },
        None => help()
    };
}

fn debug(zr_home: &Path) {
    version();
    println!("  ZR_HOME: {}", zr_home.display());
}

fn help() {
    println!(r"zr {}

usage:
  zr [<plugin>|command]


commands:
  zr load <plugin> - save 'plugin' to ~/.zr-init.zsh
  zr help - print this help
  zr version - print the version
  zr debug - print environment vars",
      VERSION);
}

fn version() {
    println!("{}", VERSION);
}

fn load(zr_home: &Path, name: &Path) {
    println!("loading {:?}", name.display());
    let plugin_path = format!("{}/{}", zr_home.display(), name.display());
    let plugin = Plugin::from_path(Path::new(&plugin_path));
    println!("loaded {:?}", name.display());
    println!("{}", plugin);
}

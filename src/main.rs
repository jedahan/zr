use std::{env,fmt};
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, ErrorKind, Write, LineWriter};
use std::collections::HashSet;

const VERSION: &'static str = "0.0.1";

#[derive(Debug)]
struct Plugin {
    name: String,
    files: Vec<PathBuf>
}

impl fmt::Display for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut basedirs = HashSet::new();
        writeln!(f, "# {}", self.name)?;
        for file in &self.files {
            basedirs.insert(file.parent().unwrap());
            writeln!(f, r"source {}", file.display())?;
        }
        for basedir in basedirs {
            writeln!(f, r"fpath+={}/", basedir.display())?;
            writeln!(f, r"PATH={}:$PATH", basedir.display())?;
        }
        Ok(())
    }
}

impl Plugin {
    pub fn from_path(path: PathBuf) -> Plugin {
        let name = String::from(path.iter().last().unwrap().to_string_lossy());
        let name_clone = &name.clone();
        let files: Vec<_> = path.read_dir().unwrap().filter_map(std::result::Result::ok).map(|file| file.path()).filter(|file| file.is_file() && file.extension().is_some()).collect();

        if let Some(antigen_plugin_file) = files.iter().find(|file| file.file_name().unwrap() == path.join(name_clone).join(".plugin.zsh")) {
            return Plugin {
                name: name,
                files: vec![antigen_plugin_file.to_owned()]
            }
        }

        // prezto: if we find init.zsh, try to load with pmodload, or manually
        if let Some(prezto_plugin_file) = files.iter().find(|file| file.file_name().unwrap() == path.join("init.zsh")) {
            return match std::process::Command::new("pmodload").arg(name.clone()).spawn() {
                Ok(_) =>
                    Plugin {
                        name: name,
                        files: vec![]
                    },
                Err(_) =>
                    Plugin {
                        name: name,
                        files: vec![prezto_plugin_file.to_owned()]
                    }
            }
        }

        // zsh plugins
        let zsh_plugin_files: Vec<_> = files.iter().filter(|file| file.extension().unwrap() == "zsh").map(|e| e.to_owned()).collect();
        if ! zsh_plugin_files.is_empty() {
            return Plugin {
                name: name,
                files: zsh_plugin_files
            }
        }

        // sh plugins
        let sh_plugin_files: Vec<_> = files.iter().filter(|file| file.extension().unwrap() == "sh").map(|e| e.to_owned()).collect();
        if ! sh_plugin_files.is_empty() {
            return Plugin {
                name: name,
                files: sh_plugin_files.to_vec()
            }
        }

        Plugin { name: name, files: vec![] }
    }
}

fn main() {
    let default_home = format!("{}/.zr", env!("HOME"));
    let zr_home = PathBuf::from(option_env!("ZR_HOME").unwrap_or(default_home.as_str()));

    match env::args().nth(1) {
        Some(command) => {
            match command.as_ref() {
                "version" => version(),
                "debug" => debug(zr_home),
                "load" => load(zr_home, PathBuf::from(env::args().nth(2).unwrap())),
                "reset" => reset(zr_home),
                _ => help(),
            }
        },
        None => help()
    };
}

fn debug(zr_home: PathBuf) {
    version();
    println!("  ZR_HOME: {}", zr_home.display());
}

fn help() {
    println!(r"zr {}

usage:
  zr [<plugin>|command]


commands:
  zr load <plugin> - save 'plugin' to ZR_HOME/init.zsh
  zr help - print this help
  zr reset - remove ZR_HOME/init.zsh
  zr version - print the version
  zr debug - print environment vars",
      VERSION);
}

fn version() {
    println!("{}", VERSION);
}

fn reset(zr_home: PathBuf) {
    let result = fs::remove_file(zr_home.join("init.zsh"));
    if result.is_err() {
        let err = result.err().unwrap();
        if err.kind() != ErrorKind::NotFound {
            Err(err).unwrap()
        }
    }
}

fn load(zr_home: PathBuf, name: PathBuf) {
    let plugin_path = PathBuf::from(format!("{}/plugins/{}", zr_home.display(), name.display()));
    let plugin = Plugin::from_path(PathBuf::from(&plugin_path));

    let init_file_path = PathBuf::from(format!("{}/init.zsh", zr_home.display()));
    if ! init_file_path.exists() {
        File::create(&init_file_path).unwrap();
    }
    let init_file = File::open(&init_file_path).unwrap();
    let buf_reader = BufReader::new(init_file);
    let all_lines = buf_reader.lines().map(|line| line.unwrap());

    let new_init_file_path = PathBuf::from(format!("{}/init.zsh.new", zr_home.display()));
    let new_init_file = File::create(&new_init_file_path).unwrap();
    let mut new_init_file = LineWriter::new(new_init_file);
    let plugin_buf = format!("{}", plugin);
    let plugin_lines = plugin_buf.lines().map(|line| line.to_string());

    let autoload_line = "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump";

    for line in all_lines.chain(plugin_lines).filter(|line| line != autoload_line) {
       new_init_file.write(line.as_bytes()).unwrap();
       new_init_file.write(b"\n").unwrap();
    }

    new_init_file.write(autoload_line.as_bytes()).unwrap();

    fs::rename(&new_init_file_path, &init_file_path).unwrap();
}

use std::fmt;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, ErrorKind, Write, LineWriter};
use std::collections::HashSet;
use std::fs::OpenOptions;

#[macro_use]
extern crate clap;

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
    /**
     * Load accepts a few kind of formats:
     *
     *     load some/repo/single.file.zsh
     */
    pub fn from_path(path: PathBuf) -> Plugin {
        let name = String::from(path.iter().last().unwrap().to_string_lossy());

        if path.is_file() {
            return Plugin {
                name: name,
                files: vec![path]
            }
        }

        let files: Vec<_> = path.read_dir().unwrap().filter_map(std::result::Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file() && file.extension().is_some())
            .collect();

        if let Some(antigen_plugin_file) = files.iter().find(|file| file.file_name().unwrap().to_string_lossy() == format!("{}.plugin.zsh", &name)) {
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

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "zsh plugin manager")
        (@subcommand version => (about: "print version") )
        (@subcommand reset => (about: "delete init.zsh") )
        (@subcommand debug => (about: "print debug info") )
        (@subcommand load =>
            (about: "load plugin")
            (@arg plugin: +required "file or folder to load")
        )
    );

    match zr.clone().get_matches().subcommand() {
        ("load", Some(load_matches)) => load(zr_home, PathBuf::from(load_matches.value_of("plugin").unwrap())),
        ("reset", _) => reset(zr_home),
        ("debug", _) => debug(zr_home),
        (_, _) => zr.print_help().unwrap()
    }
}

fn debug(zr_home: PathBuf) {
    println!("  ZR_HOME: {}", zr_home.display());
}

fn reset(zr_home: PathBuf) {
    if let Err(error) = fs::remove_file(zr_home.join("init.zsh")) {
        if error.kind() != ErrorKind::NotFound {
            Err(error).unwrap()
        }
    }
}

fn load(zr_home: PathBuf, name: PathBuf) {
    let plugin_path = PathBuf::from(format!("{}/plugins/{}", zr_home.display(), name.display()));
    let plugin = Plugin::from_path(PathBuf::from(&plugin_path));

    let init_filename = format!("{}/init.zsh", zr_home.display());
    let init_file = OpenOptions::new().read(true).write(true).create(true).open(&init_filename).unwrap();
    let init_lines = BufReader::new(&init_file).lines().map(|line| line.unwrap());

    let temp_filename = format!("{}.tmp", init_filename);
    let temp_file = OpenOptions::new().create_new(true).open(&temp_filename).unwrap();
    let mut temp_writer = LineWriter::new(temp_file);

    let plugin_buf = format!("{}", plugin);
    let plugin_lines = plugin_buf.lines().map(|line| line.to_string());

    let autoload_line = "autoload -Uz compinit; compinit -iCd $HOME/.zcompdump";

    for line in init_lines.chain(plugin_lines).filter(|line| line != autoload_line) {
       temp_writer.write(line.as_bytes()).unwrap();
       temp_writer.write(b"\n").unwrap();
    }

    temp_writer.write(autoload_line.as_bytes()).unwrap();

    fs::rename(&temp_filename, &init_filename).unwrap();
}

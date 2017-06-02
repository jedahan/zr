#![cfg_attr(feature="clippy", feature(plugin))]

#![cfg_attr(feature="clippy", plugin(clippy))]

#![feature(test)]
extern crate test;
extern crate git2;

#[macro_use]
extern crate clap;
extern crate libc;

use std::path::PathBuf;
use std::io::{BufRead, BufReader, Write};
use std::fs::OpenOptions;

mod error;
mod plugin;
mod plugins;

use plugins::*;
use error::*;

fn main() {
    if let Err(err) = run() {
        writeln!(&mut std::io::stderr(), "{}", err)
            .expect("error writing to stderr");
        std::process::exit(libc::EXIT_FAILURE);
    }
}

fn get_var(key: &str) -> Result<Option<String>, Error> {
    use std::env::VarError::*;

    match std::env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(NotPresent) => Ok(None),
        Err(NotUnicode(value)) => Err(Error::EnvironmentVariableNotUnicode { key: key.to_string(), value: value} ),
    }
}

fn load_plugins_from(zr_home: &PathBuf) -> Plugins {
    let mut plugins = Plugins::new(zr_home.clone());
    let zr_init = &zr_home.join("init.zsh");
    let plugin_home = &zr_home.join("plugins");

    if zr_init.exists() {
        let init_file = OpenOptions::new().read(true).open(&zr_init).unwrap();
        for filepath in BufReader::new(&init_file)
            .lines()
            .map(|line| line.unwrap())
            .filter(|line| line.starts_with("source"))
            .map(|line| PathBuf::from(line.split_whitespace().last().unwrap()))
            .map(|filepath| filepath.strip_prefix(&plugin_home).ok().unwrap().to_owned() )
            .collect::<Vec<_>>() {
                let filename = filepath.to_str().to_owned().unwrap();
                let name = filename.split('/').collect::<Vec<_>>()[0..2].join("/");
                let file = filename.split('/').collect::<Vec<_>>()[2..].join("/");
                let _ = plugins.add(&name, Some(&file));
            }
    }

    plugins
}

fn run() -> Result<(), Error> {
    let zr_home = get_var("ZR_HOME")?;
    let home = get_var("HOME")?;
    let default_home = format!("{}/.zr", home.unwrap());
    let path = PathBuf::from(zr_home.unwrap_or(default_home));

    let mut plugins = load_plugins_from(&path);

    let mut zr = clap_app!(zr =>
        (version: crate_version!())
        (author: "Jonathan Dahan <hi@jonathan.is>")
        (about: "z:rat: - zsh plugin manager")
        (@subcommand reset => (about: "delete init file") )
        (@subcommand list => (about: "list plugins") )
        (@subcommand update => (about: "update plugins") )
        (@subcommand add =>
            (about: "add plugin to init file")
            (@arg plugin: +required "plugin/name")
            (@arg file: "optional/path/to/file.zsh")
        )
    );

    match zr.clone().get_matches().subcommand() {
        ("add", Some(m)) => plugins.add(m.value_of("plugin").unwrap(), m.value_of("file")),
        ("list", _) => plugins.list(),
        ("reset", _) => plugins.reset(),
        ("update", _) => plugins.update(),
        (_, _) => zr.print_help().map_err(Error::Clap),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use std::env;
    use std::fs;
    use super::git2;

    #[bench]
    fn bench_add(bencher: &mut Bencher) {
        let temp_dir = env::temp_dir().join("zr");
        let path = PathBuf::from(temp_dir);
        if path.is_dir() {
            fs::remove_dir_all(&path).unwrap();
        }
        fs::create_dir_all(&path).unwrap();
        // clone all the repos here

        let plugins = vec![
            "zsh-users/prezto",
            "junegunn/fzf",
            "zsh-users/zsh-autosuggestions",
            "zdharma/fast-syntax-highlighting",
            "zsh-users/zsh-history-substring-search",
            "molovo/tipz",
            "changyuheng/zsh-interactive-cd",
            "frmendes/geometry",
            "jedahan/geometry-hydrate"
        ];
        for plugin in plugins {
            let plugin_path = path.join(&plugin);
            let plugin_parent = plugin_path.parent().unwrap();
            let _ = fs::create_dir(&plugin_parent);
            let url = format!("https://github.com/{}", plugin);
            git2::Repository::clone(&url, &plugin_path).unwrap();
        }

        let mut plugins = load_plugins_from(&path);

        bencher.iter(|| {
            let _ = plugins.add("zsh-users/prezto", Some("modules/git/alias.zsh"));
            let _ = plugins.add("zsh-users/prezto", Some("modules/history/init.zsh"));
            let _ = plugins.add("zsh-users/prezto", Some("modules/osx/init.zsh"));
            let _ = plugins.add("junegunn/fzf", Some("shell/key-bindings.zsh"));
            let _ = plugins.add("zsh-users/zsh-autosuggestions", Some(""));
            let _ = plugins.add("zdharma/fast-syntax-highlighting", Some(""));
            let _ = plugins.add("zsh-users/zsh-history-substring-search", Some(""));
            let _ = plugins.add("molovo/tipz", Some(""));
            let _ = plugins.add("changyuheng/zsh-interactive-cd", Some(""));
            let _ = plugins.add("frmendes/geometry", Some(""));
            let _ = plugins.add("jedahan/geometry-hydrate", Some(""));
        } );

        use std::io::BufReader;
        use std::fs::OpenOptions;
        let init_path = path.join("init.zsh");
        let init_file = OpenOptions::new().read(true).open(init_path).unwrap();
        let lines = BufReader::new(init_file).lines();
        let count = lines.count();
        assert_eq!(52, count);
    }
}

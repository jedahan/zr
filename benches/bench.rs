#![feature(test)] extern crate test;
extern crate git2;
extern crate libzr as zr;

use std::path::PathBuf;
use std::io::BufRead;

use test::Bencher;
use std::env;
use std::fs;

fn bench_run(bencher: &mut Bencher) {
    bencher.iter(|| {
        zr::run()
    });
}

fn clone_repos() -> PathBuf {
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

    path
}

fn test_52_lines(path: PathBuf) {
    use std::io::BufReader;
    use std::fs::OpenOptions;
    let init_path = path.join("init.zsh");
    let init_file = OpenOptions::new().read(true).open(init_path).unwrap();
    let lines = BufReader::new(init_file).lines();
    let count = lines.count();
    assert_eq!(52, count);
}

#[bench]
fn bench_load(bencher: &mut Bencher) {
    let path = clone_repos();

     let plugins: Vec<String> = vec![
         "zsh-users/prezto/modules/git/alias.zsh",
         "zsh-users/prezto/modules/history/init.zsh",
         "zsh-users/prezto/modules/osx/init.zsh",
         "junegunn/fzf/shell/key-bindings.zsh",
         "zsh-users/zsh-autosuggestions",
         "zdharma/fast-syntax-highlighting",
         "zsh-users/zsh-history-substring-search",
         "molovo/tipz",
         "changyuheng/zsh-interactive-cd",
         "frmendes/geometry",
         "jedahan/geometry-hydrate"
     ].iter().map(std::string::ToString::to_string).collect();

    bencher.iter(|| {
        let _ = zr::load_plugins(&path, plugins.clone());
    } );
}

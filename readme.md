### z :rat:

<img src="zrat.png" alt="zrat" title="zrat" align="right" width=200 />

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://choosealicense.com/licenses/mpl-2.0/)
[![Build Status](https://travis-ci.org/jedahan/zr.svg?branch=master)](https://travis-ci.org/jedahan/zr)

Quick, simple zsh plugin manager

    zr 0.6.6
    Jonathan Dahan <hi@jonathan.is>
    z:rat: - zsh plugin manager

    USAGE:
        zr [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        help      Prints this message or the help of the given subcommand(s)
        list      list plugins
        load      generate init file from plugin list
        update    update plugins

#### install

You can use crates.io and cargo to just `cargo install zr`

#### usage

Add this to your *~/.zshrc*:

```zsh
# Generate new ~/.zr/init.zsh if it does not exist or ~/.zshrc is newer
if [[ ! -f ~/.zr/init.zsh ]] || [[ ~/.zshrc -nt ~/.zr/init.zsh ]]; then
  test -d ~/.zr || mkdir $_
  zr load \
    frmendes/geometry \
    jedahan/geometry-hydrate \
    junegunn/fzf/shell/key-bindings.zsh  # just load key-bindings.zsh
fi

source ~/.zr/init.zsh
```

If you'd like a different directory for `~/.zr`, just set `ZR_HOME`

#### speed

The following __[init.zsh][]__ takes 180ms total, with 26ms spent in `zr load` for my 2012 13" retina macbook pro.

See [the wiki](https://github.com/jedahan/zr/wiki) for more details.

```zsh
# cargo install hyperfine
# hyperfine --warmup 3 'zsh -d -f -l -c "source benchmark.zsh && zrinit && exit"'
Time (mean ± σ):      26.0 ms ±   4.6 ms
```

```zsh
# benchmark.zsh
function zrinit {
  zr load sorin-ionescu/prezto/modules/git/alias.zsh \
    sorin-ionescu/prezto/modules/history/init.zsh \
    junegunn/fzf/shell/key-bindings.zsh \
    zsh-users/zsh-autosuggestions \
    zdharma/fast-syntax-highlighting \
    molovo/tipz \
    geometry-zsh/geometry \
    jedahan/geometry-hydrate \
    jedahan/geometry-todo \
    geometry-zsh/geometry \
    ael-code/zsh-colored-man-pages \
    momo-lab/zsh-abbrev-alias \
    jedahan/alacritty-completions \
    zpm-zsh/ssh
}
```

#### thanks

- [SX91](https://github.com/SX91) for linux fixes
- [alanpearce](https://github.com/alanpearce) for bug reports and nix package
- [nshtg](https://github.com/nshtg) for bug reports and windows fix

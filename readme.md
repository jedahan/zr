### z :rat:

<img src="zrat.png" alt="zrat" title="zrat" align="right" width=200 />

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://choosealicense.com/licenses/mpl-2.0/)
[![Build Status](https://travis-ci.org/jedahan/zr.svg?branch=master)](https://travis-ci.org/jedahan/zr)

Quick, simple zsh plugin manager

    zr 0.9.0
    by Jonathan Dahan <hi@jonathan.is>

    zr [[http://example.com]plugin/name[.git/path/to/file.zsh]]    fetch or update plugins and output sourceable zsh
    zr help     show help

#### install

You can use crates.io and cargo to just `cargo install zr`

#### usage

Add this to your *~/.zshrc*:

```zsh
# Generate new ~/.zr/init.zsh if it does not exist or ~/.zshrc is newer
if [[ ! -f ~/.config/zr.zsh ]] || [[ ~/.zshrc -nt ~/.config/zr.zsh ]]; then
  zr load \
    frmendes/geometry \
    jedahan/geometry-hydrate \
    junegunn/fzf/shell/key-bindings.zsh \
    > ~/.config/zr.zsh
fi

source ~/.config/zr.zsh
```

#### identifiers

zr supports four identifier formats, note that the last format requires `.git` as a delimeter.

format                                     | resolves to
-------------------------------------------|-----------
`author/name`                              | __*.zsh__ from https://github.com/author/name
`author/name/file.zsh`                     | __file.zsh__ from https://github.com/author/name
`https://gitlab.com/a/plugin`              | __*.zsh__ from https://gitlab.com/a/plugin
`https://gitlab.com/a/plugin.git/file.zsh` | __file.zsh__ from https://gitlab.com/a/plugin.git. The `.git` is used as a delimeter, and is required.

#### speed

The following benchmark.zsh takes 171ms total, with 26ms spent in `zr load` for my 2012 13" retina macbook pro.

```zsh
# install hyperfine for benchmarking
cargo install hyperfine

# benchmark zr init time
hyperfine --warmup 3 'zsh -d -f -l -c "source benchmark.zsh && zrinit && exit"' | grep Time
Time (mean ± σ):      26.0 ms ±   1.6 ms

# benchmark total zsh load time
hyperfine --warmup 3 'zsh -d -f -l -c "source benchmark.zsh && zrinit && source ~/.zr/init.zsh && exit"' | grep Time
Time (mean ± σ):     171.6 ms ±  4.4 ms
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
- [foray1010](https://github.com/foray1010) for improving install instructions
- everyone on [#rust-beginners](irc://irc.mozilla.org/rust-beginners)

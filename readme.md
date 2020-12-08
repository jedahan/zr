### z :rat:

<img src="zrat.png" alt="zrat" title="zrat" align="right" width=200 />

[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://choosealicense.com/licenses/mpl-2.0/)
[![Build Status](https://travis-ci.org/jedahan/zr.svg?branch=master)](https://travis-ci.org/jedahan/zr)

Quick, simple zsh plugin manager

    zr 1.0.0
    by Jonathan Dahan <github@jonathan.is>

    zr --update                                                    updates loaded files listened in _ZR environment variable
    zr [[http://example.com]plugin/name[.git/path/to/file.zsh]]    fetch or update plugins and output sourceable zsh
    zr help                                                        show help

#### install

You can use crates.io and cargo to just `cargo install zr`

#### usage

Add this to your *~/.zshrc*:

```zsh
# Generate new ~/.zr/init.zsh if it does not exist or ~/.zshrc is newer
if [[ ! -f ~/.config/zr.zsh ]] || [[ ~/.zshrc -nt ~/.config/zr.zsh ]]; then
  zr \
    frmendes/geometry \
    jedahan/geometry-hydrate \
    junegunn/fzf.git/shell/key-bindings.zsh \
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

The following two benchmarks show on my dell xps13 9380
* it takes 5ms to generate a sourceable script from a dozen or so repos
* it takes an additional 15ms for zsh to load said script

```zsh
# install hyperfine for benchmarking
$ which hyperfine || cargo install hyperfine

# run 
$ hyperfine --warmup 3 'zsh -d -f -l -c "source benchmark.zsh && zrinit && exit"' 'zsh -d -f -l -c "source benchmark.zsh && . <(zrinit) && exit"'

Benchmark #1: zsh -d -f -l -c "source benchmark.zsh && zrinit && exit"
  Time (mean ± σ):       5.3 ms ±   2.3 ms    [User: 2.8 ms, System: 2.4 ms]
  Range (min … max):     2.9 ms …   9.9 ms    285 runs

Benchmark #2: zsh -d -f -l -c "source benchmark.zsh && . <(zrinit) && exit"
  Time (mean ± σ):      21.8 ms ±   1.0 ms    [User: 17.5 ms, System: 5.1 ms]
  Range (min … max):    19.7 ms …  26.4 ms    127 runs
```

```zsh
# benchmark.zsh
function zrinit {
  XDG_CACHE_HOME=/tmp/zrbenchmark zr sorin-ionescu/prezto.git/modules/git/alias.zsh \
    sorin-ionescu/prezto.git/modules/history/init.zsh \
    junegunn/fzf.git/shell/key-bindings.zsh \
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
- [Avi-D-coder](https://github.com/avi-d-coder) for adding completions support
- [@ralsei](https://github.com/ralsei) for prodding to update crates.io
- [@tekumara](https://github.com/tekumara) for helping figure out --update
- everyone on [#rust-beginners](irc://irc.mozilla.org/rust-beginners)

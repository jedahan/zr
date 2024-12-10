<img src="zrat.png" alt="zrat" title="zrat" width=120 />

### zr(at)

nimble zsh plugin manager

    zr 1.2.1
    by Jonathan Dahan <hi@jonathan.is>

    Example

    . <(zr geometry-zsh/geometry junegunn/fzf.git/shell/key-bindings.zsh)

    Format

    zr author/name                                    *.zsh from github.com/author/name
    zr author/name/file.zsh                        file.zsh from github.com/author/name
    zr https://gitlab.com/a/plugin                    *.zsh from gitlab.com/a/plugin
    zr https://gitlab.com/a/plugin.git/file.zsh    file.zsh from gitlab.com/a/plugin

    Commands

    zr +update                                      update plugins from already sourced zsh
    zr +help                                        show help

#### install

`zr` is published to crates.io, and can be installed with `cargo install zr`

#### usage

Add this to your *~/.zshrc*:

```zsh
. <(zr frmendes/geometry junegunn/fzf.git/shell/key-bindings.zsh)
```

A bit more complex example, that only generates when .zshrc has been updated:

```zsh
# Generate new ~/.config/zr.zsh if it does not exist or if ~/.zshrc has been changed
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

`zr` supports four identifier formats. The last format requires `.git` as a delimeter.

format                                     | resolves to
-------------------------------------------|-----------
`author/name`                              | __*.zsh__ from https://github.com/author/name
`author/name/file.zsh`                     | __file.zsh__ from https://github.com/author/name
`https://gitlab.com/a/plugin`              | __*.zsh__ from https://gitlab.com/a/plugin
`https://gitlab.com/a/plugin.git/file.zsh` | __file.zsh__ from https://gitlab.com/a/plugin.git. The `.git` is used as a delimeter, and is required.

#### speed

The following two benchmarks show on a 2024 14" macbook pro m4
* it takes 10ms to generate a sourceable script from a dozen or so repos
* it takes an additional 30ms for zsh to load said script

```zsh
$ hyperfine --warmup 3 \
  --command-name "generate shell script" \
    'zsh -d -f -l -c "source benchmark.zsh && zrinit && exit"' \
  --command-name "generate and load shell script" \
    'zsh -d -f -l -c "source benchmark.zsh && . <(zrinit) && exit"'

Benchmark 1: generate shell script
  Time (mean ± σ):       8.2 ms ±   1.3 ms    [User: 3.4 ms, System: 3.7 ms]
  Range (min … max):     7.2 ms …  12.1 ms    258 runs

Benchmark 2: generate and load shell script
  Time (mean ± σ):      42.1 ms ±   2.6 ms    [User: 22.4 ms, System: 58.2 ms]
  Range (min … max):    38.7 ms …  51.2 ms    54 runs
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
- [ralsei](https://github.com/ralsei) for prodding to update crates.io
- [tekumara](https://github.com/tekumara) for helping figure out --update
- [myrovh](https://github.com/myrovh) for fixing panics on some linux systems
- [TimB87](https://github.com/TimB87) for adding openssl 3 support
- [olets](https://github.com/olets) for adding recursive clone support
- everyone on [#rust-beginners](irc://irc.mozilla.org/rust-beginners)

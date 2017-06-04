# z :rat:

    zr 0.4.5
    Jonathan Dahan <hi@jonathan.is>
    z:rat: - zsh plugin manager

    USAGE:
        zr [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        add       add plugin to init file
        help      Prints this message or the help of the given subcommand(s)
        list      list plugins
        load      load all plugins in a single command
        reset     delete init file
        update    update plugins


# usage

Add this to your *~/.zshrc*:

```zsh
test -f ~/.zr/init.zsh || {
  zr load \
  frmendes/geometry \
  jedahan/geometry-hydrate \
  junegunn/fzf shell/key-bindings.zsh  # just load key-bindings.zsh
}

source ~/.zr/init.zsh
```

After adding or removing plugins, run `zr reset`

If you'd like a different directory for `~/.zr`, just set `ZR_HOME`

# speed

Resetting and generating the following __[init.zsh][]__ adds 0.08s on top of 0.21s load time for my 2012 13" retina macbook pro.
See [the wiki](https://github.com/jedahan/zr/wiki) for more details.

    zr load \
    zsh-users/prezto modules/git/alias.zsh \
    zsh-users/prezto modules/osx/init.zsh \
    zsh-users/prezto modules/history/init.zsh \
    junegunn/fzf shell/key-bindings.zsh \
    zsh-users/zsh-autosuggestions \
    zdharma/fast-syntax-highlighting \
    zsh-users/zsh-history-substring-search \
    molovo/tipz \
    changyuheng/zsh-interactive-cd \
    frmendes/geometry \
    jedahan/geometry-hydrate\
    source ~/.zr/init.zsh

## benchmarks

    cargo +nightly bench

# developing

    cargo +nightly build --features="clippy"

[init.zsh]: https://github.com/jedahan/dotfiles/blob/master/.zshrc

# thanks

- [SX91](https://github.com/SX91) for linux fixes

# z :rat:

    zr 0.4.2
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
        reset     delete init file
        update    update plugins


# usage

Add this to your *~/.zshrc*:

```zsh
test -f $HOME/.zr/init.zsh && source $_ || {
  zr add frmendes/geometry
  zr add jedahan/geometry-hydrate
  zr add junegunn/fzf shell/key-bindings.zsh  # just load key-bindings.zsh
  exec zsh
}
```

When you want to add or remove plugins, just run `zr reset`

If you'd like a different directory for `~/.zr`, just set `ZR_HOME`

# speed

Resetting, generating and loading the following _[init.zsh][]_ adds 0.15s on my 2012 13" retina macbook pro (going from 0.20s to 0.35s).
See [the wiki](/wiki) for more details.

    zr reset                                      # remove ~/.zr/init.zsh
    zr add zsh-users/prezto modules/git/alias.zsh # sensible git aliases
    zr add zsh-users/prezto modules/osx/init.zsh  # some osx shortcuts
    zr add junegunn/fzf shell/key-bindings.zsh    # fuzzy finder, try ^r, ^t, kill<tab>
    zr add zsh-users/zsh-autosuggestions          # suggest from history
    zr add zdharma/fast-syntax-highlighting       # commandline syntax highlighting
    zr add zsh-users/zsh-history-substring-search # partial fuzzy history search
    zr add molovo/tipz                            # help remember aliases
    zr add changyuheng/zsh-interactive-cd         # tab complete fuzzy finder cd
    zr add frmendes/geometry                      # clean theme
    zr add jedahan/geometry-hydrate               # remind you to hydrate
    source ~/.zr/init.zsh

# developing

    cargo +nightly build --features="clippy"

[init.zsh]: https://github.com/jedahan/dotfiles/blob/master/.zshrc

# thanks

- [SX91](https://github.com/SX91) for contribute fixes for linux

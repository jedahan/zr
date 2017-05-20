# z :rat:

    zr 0.4.1
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

# developing

    cargo +nightly build --features="clippy"

# z :rat:

    zr 0.4.0
    Jonathan Dahan <hi@jonathan.is>
    z:rat: - zsh plugin manager

    USAGE:
        zr [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        add       add plugin to init file
        debug     print debug information
        help      Prints this message or the help of the given subcommand(s)
        list      list plugins
        reset     delete init file
        update    update plugins


# usage

Add this to your *~/.zshrc*:

    test -f $HOME/.zr/init.zsh && source $_ || {
      zr add frmendes/geometry
      zr add jedahan/geometry-hydrate
      exec zsh
    }

When you want to add or remove plugins, just run `zr reset`

# developing

    cargo +nightly build --features="clippy"

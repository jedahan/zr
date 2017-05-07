# z :rat:

    zr 0.1.0
    Jonathan Dahan <hi@jonathan.is>
    zsh plugin manager

    USAGE:
        zr [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        debug      print debug info
        help       Prints this message or the help of the given subcommand(s)
        load       load plugin
        reset      delete init.zsh
        version    print version

# developing

    cargo +nightly build --features="clippy"

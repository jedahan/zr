# 0.5.0 unreleased

+ Added output when cloning repositories
+ Added sample zshrc benchmark
- Deprecated `add` and `reset`, as `load` is fast enough, and you can regen when mtime is different via

    [[ ~/.zshrc -nt ~/.zr/init.zsh ]] && { zr load \ ... }

# 0.4.6 fink

Added `load` command, which is about twice as fast as generating init.zsh.
If you had a *zshrc* that looked like:

    zr add some/plugin
    zr add other/plugin some/file.zsh

You can migrate to

    zr load \
      some/plugin \
      other/plugin some/file.zsh

Which will generate everything in one go.

This might be fast enough to always regenerate init.zsh on shell load.

Also added benchmarks, which can be run on nightly now.

# 0.4.3 neotoma

This is the first public release of zr!

Thank you to [SX91](https://github.com/SX91) for contributing fixes for linux.

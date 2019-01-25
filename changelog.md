# 0.7.1

* Removed extraneous `plugins` directory

# 0.7.0

We finally have support for arbitrary git sources!
It is recommended to delete your `ZR_HOME` directory.

If you are cloning from non-github repo, and specifying the plugin path, you must have `.git` as a separator so `zr` knows what part of the path points to the git repo, and what points to the file.
See the readme for examples.

+ added --home, -h config flag to specify home
+ added non-github repository support

    zr supports https://whatever.com/some/repo[.git/path/to/files.zsh]

- Breaking change: plugins are cloned into top-level `ZR_HOME` (no author/name subdirectories)
- Breaking change: `zr list` now shows full identifier for sources (no author/name shortname)

# 0.6.6

+ Update to rust 2018

# 0.6.5

+ Fix path generation on windows (thanks @nshtg!)

# 0.6.4

+ Update dependencies
+ Minor cleanup

# 0.6.3

+ Update dependencies

# 0.6.2

+ License under the MPL-2.0
+ Publish to crates.io

# 0.6.1

+ Create supporting directories if they do not exist

# 0.6.0 sabanus

- Breaking change: `load` now requires the path to be part of the plugin definition

    zr load author/plugin some/file.zsh

Now must be

    zr load author/plugin/some/file.zsh

This fixes a lot of brittleness

# 0.5.0

+ Added sample zshrc benchmark
- Removed `add` and `reset`

# 0.4.8

+ Added output when cloning repositories
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

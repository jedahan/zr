# 1.1.0
* update dependencies to support openssl 3 (thanks @TimB87)

# 1.0.2
* cleanup help output

# 1.0.1
* add `functions` directories to fpath if functions are found
* updated dependencies

# 1.0.0
* Finally released!
* :warning: if you are updating from 0.8.2 (last published crates.io version) this will break your config!
  * read the changelog for 0.9.0 on how to update
* Thank you @ralsei and @tekumara

# 0.9.0

* Deprecated `load`, `update`, and `list`
* Write to stdout instead of ~/.config/zr
* Use XDG_CACHE_DIR/zr or $HOME/.cache/zr for caching plugins

* Added support for completions, thanks @Avi-D-coder!

* Strip binary, reducing size from 4.9Mb to 1.8Mb
* Removed dependency on clap, reducing size by 700kb

# 0.8.2

* Support git ssh credentials during updates!

# 0.8.1

* Only add a directory to PATH and fpath if there are executable files inside (non-windows specific)

# 0.8.0

- Removed `-h` shorthand for `--home`, now `-h` is for `--help`
* Bumped minor version of deps

# 0.7.2

* Bumped patch version of deps

# 0.7.1

+ Added some documentation which can be seen with `cargo doc -p zr --open`.
* Removed extraneous `plugins` directory
* Bumped minor version of libc crate

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

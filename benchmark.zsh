# hyperfine --warmup 3 'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit && exit"'
function zrinit {
  XDG_CACHE_HOME=/tmp/zrbench ${1:-target/release/zr} \
    sorin-ionescu/prezto.git/modules/git/alias.zsh \
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

# For comparing different release profiles

# hyperfine --warmup 3 \
#  'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit target/release/zr-0 && exit"' \
#  'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit target/release/zr-s && exit"' \
#  'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit target/release/zr-3 && exit"' \
#  'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit target/release/zr-lto && exit"' \
#  'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit target/release/zr-default && exit"'

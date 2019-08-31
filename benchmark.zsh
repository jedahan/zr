# hyperfine --warmup 3 'zsh -d -f -l -c "source $PWD/benchmark.zsh && zrinit && exit"'
function zrinit {
  zr --home /tmp/zrbench load sorin-ionescu/prezto/modules/git/alias.zsh \
    sorin-ionescu/prezto/modules/history/init.zsh \
    junegunn/fzf/shell/key-bindings.zsh \
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

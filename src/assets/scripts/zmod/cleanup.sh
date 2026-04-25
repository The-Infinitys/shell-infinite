#!/bin/zsh

# 1. フックの解除
autoload -Uz add-zsh-hook
add-zsh-hook -d precmd _zsh_infinite_precmd

# 2. ZLEウィジェットの復元
if zle -l zle-line-finish; then
    # 保存していた元のウィジェット名を取得
    local original_widget
    original_widget=$(__zsh_infinite_internal get zle-line-finish 2>/dev/null)
    
    # 現在のカスタムウィジェットを削除
    zle -D zle-line-finish
    
    # 元のウィジェットが存在していれば再登録
    if [[ -n "$original_widget" && "$original_widget" != "builtin" ]]; then
        zle -N zle-line-finish "$original_widget"
    fi
fi

if zle -l interrupt; then
    zle -D interrupt
fi

# 3. 関数と環境変数の削除
unfunction _zsh_infinite_precmd
unfunction _zle_infinite_line_finish
unfunction _zle_infinite_interrupt
unset ZLE_RPROMPT_INDENT

# 4. オプションを戻す（必要に応じて）
setopt PROMPT_SUBST

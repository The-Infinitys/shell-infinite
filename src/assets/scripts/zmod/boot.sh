#!/bin/zsh
unsetopt PROMPT_SUBST
# パターンマッチに失敗してもエラーを出さず、そのまま文字列として扱う
setopt no_nomatch
ZLE_RPROMPT_INDENT=0


# --- プロンプト更新フック ---
function _zsh_infinite_precmd() {
    __zsh_infinite_internal precmd 2>/dev/null
    echo -ne '\e[0 q'
}

# --- Transient Prompt ---
function _zle_infinite_line_finish() {
    __zsh_infinite_internal line-finish 2>/dev/null
}

function _zle_infinite_interrupt() {
    __zsh_infinite_internal interrupt 2>/dev/null
    zle .interrupt
}

{
    autoload -Uz add-zsh-hook
    add-zsh-hook precmd _zsh_infinite_precmd
    local current_widget="${widgets[zle-line-finish]}"
    local old_func=""

    if [[ "$current_widget" == "user:"* ]]; then
        # "user:_original_func" から関数名部分だけを抽出
        old_func="${current_widget#user:}"
    fi
    __zsh_infinite_internal store zle-line-finish "${old_func:-${current_widget}}"
    zle -N zle-line-finish _zle_infinite_line_finish
    zle -N interrupt _zle_infinite_interrupt
} >/dev/null 2>&1

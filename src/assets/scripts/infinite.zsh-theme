#!/bin/zsh

# --- 1. 環境判定とライブラリロードの試行 ---
# VS Codeではない、かつ libzsh_infinite が利用可能な場合はライブラリモードに移行
if [[ "ZSH_INFINITE_BINMODE" != "true" && "$TERM_PROGRAM" != "vscode" && "$TERMINAL_EMULATOR" != "JetBrains-JediTerm" ]]; then
    # モジュールパスの追加（重複を避けるため ${(U)} でユニーク化）
    module_path=($HOME/.local/lib/ $module_path)
    
    # zmodload が成功すれば、このスクリプトの残りの処理（バイナリ版の設定）をスキップ
    # ※ライブラリ側で必要なフックがすべて登録される前提です
    if zmodload libzsh_infinite 2>/dev/null; then
        return 0
    fi
fi

# --- 2. バイナリ呼び出しモードの設定（VS Code または ライブラリ不在時） ---
# プロンプトの置換を有効化
setopt PROMPT_SUBST
# 右プロンプトの右端の空白を詰める
ZLE_RPROMPT_INDENT=0

# プロンプト更新用のメイン関数
function _update_infinite_prompt() {
    local last_status=$?
    
    # --- カーソルリセットを実行 ---
    _reset_cursor
    # サブシェル内での実行により、VS Codeの解析との干渉を防ぐ
    PROMPT='$(LAST_COMMAND_EXECUTED='$LAST_COMMAND_EXECUTED' LAST_STATUS='${last_status}' {{RUN_DIR}}/zsh-infinite zsh prompt left 2>/dev/null)'
    RPROMPT='$(LAST_COMMAND_EXECUTED='$LAST_COMMAND_EXECUTED' LAST_STATUS='${last_status}' {{RUN_DIR}}/zsh-infinite zsh prompt right 2>/dev/null)'
}

# コマンド確定時（エンターキー押下時）の処理
function _infinite_transient_prompt() {
    local last_status=$?
    export LAST_COMMAND_EXECUTED=$EPOCHREALTIME
    PROMPT='$( {{RUN_DIR}}/zsh-infinite zsh prompt transient --exit-code='${last_status}' 2>/dev/null)'
    RPROMPT=''
    zle reset-prompt    
}

# カーソル形状をデフォルト（ブロック等）に戻す
function _reset_cursor() {
    echo -ne '\e[0 q'
}

# フックの登録
autoload -Uz add-zsh-hook
add-zsh-hook precmd _update_infinite_prompt

# Transient Prompt 用のウィジェット登録
zle -N zle-line-finish _infinite_transient_prompt

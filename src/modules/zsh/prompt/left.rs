use crate::{
    prompt_theme,
    zsh::prompt::{Prompt, PromptConnection, PromptCurveLine},
};
use crossterm::terminal;
use futures::future::join_all;
use unicode_width::UnicodeWidthStr;
use zsh_seq::{NamedColor, ZshPromptBuilder};

pub async fn left() -> ZshPromptBuilder {
    let theme = prompt_theme();
    let mut builder = ZshPromptBuilder::new();
    if theme.prompt_contents_list.is_empty() {
        // デフォルトのPromptContentsから設定を取得
        let default_prompt_contents = crate::zsh::theme::prompt_theme::PromptContents::default();
        let curved_lines = PromptCurveLine::from(default_prompt_contents.connection);
        let h = &curved_lines.horizontal;
        return ZshPromptBuilder::new()
            .color(default_prompt_contents.color.sc)
            .str(&curved_lines.top_left)
            .str(h)
            .str(h)
            .str(&curved_lines.top_right)
            .end_color()
            .color(default_prompt_contents.color.sc)
            .str(&curved_lines.bottom_left)
            .str(h)
            .str(" ")
            .end_color();
    }

    // 2. リストがある場合のメインループ
    for (i, prompt_contents) in theme.prompt_contents_list.iter().enumerate() {
        let mut prompt = Prompt::default();
        let curved_lines = PromptCurveLine::from(prompt_contents.connection);
        let h = &curved_lines.horizontal;

        // (非同期取得部分は変更なし)
        let left_futures: Vec<_> = prompt_contents
            .left
            .iter()
            .map(|c| async move { c.content().await })
            .collect();
        let right_futures: Vec<_> = prompt_contents
            .right
            .iter()
            .map(|c| async move { c.content().await })
            .collect();
        let (left_results, right_results) =
            tokio::join!(join_all(left_futures), join_all(right_futures));
        let left_results = left_results.into_iter().filter(|r| !r.is_empty()).collect();
        let right_results = right_results
            .into_iter()
            .filter(|r| !r.is_empty())
            .collect();
        prompt.extend_left(left_results);
        prompt.extend_right(right_results);

        let left_content = prompt.render_left(prompt_contents);
        let right_content = prompt.render_right(prompt_contents);
        let terminal_width = std::env::var("COLUMNS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| terminal::size().map(|(w, _)| w).unwrap_or(80) as usize);
        let left_width = left_content.len();
        let right_width = right_content.len();
        let conn_line_width =
            UnicodeWidthStr::width(prompt_contents.connection.to_string().as_str());
        let side_decor_width =
            UnicodeWidthStr::width(curved_lines.top_left.as_str()) + conn_line_width;

        let stuff_width = left_width + right_width + side_decor_width * 2;
        let connection_len = if stuff_width < terminal_width {
            terminal_width - stuff_width
        } else {
            (terminal_width * 2).saturating_sub(stuff_width) % terminal_width
        };
        let connection_str = prompt_contents // `theme.connection` から `prompt_contents.connection` に変更
            .connection
            .to_string()
            .repeat(connection_len / conn_line_width);
        let mut row_builder = ZshPromptBuilder::new();
        row_builder = row_builder.color(prompt_contents.color.sc); // `theme.color.sc` から `prompt_contents.color.sc` に変更

        // 最初の行は TopLeft、それ以外は CrossLeft
        if i == 0 {
            row_builder = row_builder.str(&curved_lines.top_left);
        } else {
            row_builder = row_builder.str(&curved_lines.cross_left);
        }

        let final_prompt = row_builder
            .str(h)
            .end_color()
            .connect(left_content)
            .color(prompt_contents.color.pc) // `theme.color.pc` から `prompt_contents.color.pc` に変更
            .str(&connection_str)
            .end_color()
            .connect(right_content)
            .color(prompt_contents.color.sc) // `theme.color.sc` から `prompt_contents.color.sc` に変更
            .str(h)
            .str(if i == 0 {
                &curved_lines.top_right
            } else {
                &curved_lines.cross_right
            })
            .end_color();
        builder = builder.connect(final_prompt).newline();
    }
    let (sc, connection) = match theme.prompt_contents_list.last() {
        Some(contents) => (contents.color.sc, contents.connection),
        None => (NamedColor::LightBlack, PromptConnection::default()),
    };
    let curved_lines = PromptCurveLine::from(connection);
    let h = &curved_lines.horizontal;
    let end = ZshPromptBuilder::new()
        .color(sc)
        .str(&curved_lines.bottom_left)
        .str(h)
        .str(" ")
        .end_color();
    builder.connect(end)
}

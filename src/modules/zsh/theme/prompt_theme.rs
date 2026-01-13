use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};
use tokio::process::Command;
use zsh_seq::{NamedColor, ZshSequence};

use super::color_scheme::PromptColorScheme;
// 変更
use crate::zsh::{
    prompt::{PromptConnection, PromptSeparation},
    theme::color_named_color::ToNamedColor,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default, Copy)]
pub enum AccentWhich {
    #[default]
    ForeGround,
    BackGround,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptTheme {
    pub prompt_contents_list: Vec<PromptContents>,
    #[serde(default)]
    pub transient_color: PromptColorScheme,
}

impl Default for PromptTheme {
    fn default() -> Self {
        Self {
            prompt_contents_list: vec![PromptContents::default()],
            transient_color: PromptColorScheme::transient(),
        }
    }
}
impl PromptTheme {
    pub fn infinite() -> Self {
        Self {
            prompt_contents_list: vec![PromptContents::infinite()],
            transient_color: PromptColorScheme::transient(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PromptSegmentSeparators {
    pub start_separator: PromptSeparation,
    pub mid_separator: PromptSeparation,
    pub end_separator: PromptSeparation,
    pub edge_cap: bool,
    pub bold_separation: bool,
}

impl Default for PromptSegmentSeparators {
    fn default() -> Self {
        Self {
            start_separator: PromptSeparation::Sharp,
            mid_separator: PromptSeparation::Sharp,
            end_separator: PromptSeparation::Sharp,
            edge_cap: true,
            bold_separation: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptContents {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub left: Vec<PromptContent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right: Vec<PromptContent>,
    #[serde(default)]
    pub color: super::color_scheme::PromptColorScheme,
    pub connection: PromptConnection,
    pub left_segment_separators: PromptSegmentSeparators,
    pub right_segment_separators: PromptSegmentSeparators,
    pub accent_which: AccentWhich,
}

impl Default for PromptContents {
    fn default() -> Self {
        Self {
            left: vec![
                // 以前の whoami を Shell で実装
                PromptContent::Shell {
                    cmd: "whoami".to_string(),
                    args: vec![],
                    envs: HashMap::new(),
                    fg: None,
                    bg: None,
                },
                // hostname を Shell で実装
                PromptContent::Shell {
                    cmd: "hostname".to_string(),
                    args: vec![],
                    envs: HashMap::new(),
                    fg: None,
                    bg: None,
                },
            ],
            right: vec![
                // ディレクトリ表示 (PWD) は BuildIn もしくは Daemon の Pwd コマンドを利用
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Pwd { color: None },
                },
                // 終了コードの表示。Cmd コマンドを利用（環境変数は呼び出し側で解決）
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Cmd {
                        last_status: "$LAST_STATUS".to_string(),
                        last_command_executed: None,
                        color: None,
                    },
                },
            ],
            color: super::color_scheme::PromptColorScheme::default(),
            connection: PromptConnection::default(),
            left_segment_separators: PromptSegmentSeparators::default(),
            right_segment_separators: PromptSegmentSeparators::default(),
            accent_which: AccentWhich::default(),
        }
    }
}
impl PromptContents {
    pub fn infinite() -> Self {
        use zsh_prompts::Color;
        Self {
            left: vec![
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Os {
                        color: Some("white".to_string()),
                    },
                },
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Pwd {
                        color: Some("#00FFFF".to_string()),
                    },
                },
            ],
            right: vec![
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Cmd {
                        last_status: "$LAST_STATUS".to_string(),
                        last_command_executed: Some("$LAST_COMMAND_EXECUTED".to_string()),
                        color: None,
                    },
                },
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Git {
                        path: None,
                        options: zsh_prompts::git::GitStatusOptions {
                            default_color_option: None,
                            git_icon_color_option: Some(Color::Blue),
                            branch_color_option: Some(Color::White),
                            staged_color_option: Some(Color::Cyan),
                            unstaged_color_option: Some(Color::Red),
                            untracked_color_option: Some(Color::Rgb(255, 0, 112)),
                            conflict_color_option: Some(Color::Magenta),
                            stashed_color_option: Some(Color::Black),
                            clean_color_option: Some(Color::White),
                            ahead_color_option: Some(Color::Green),
                            behind_color_option: Some(Color::Red),
                        },
                    },
                },
                PromptContent::BuildIn {
                    command: zsh_prompts::Commands::Time {
                        color: Some("green".to_string()),
                    },
                },
            ],
            color: super::color_scheme::PromptColorScheme {
                bg: NamedColor::FullColor((32, 32, 32)),
                fg: NamedColor::White, // もしくは NamedColor::White
                pc: NamedColor::Red,
                sc: NamedColor::LightBlack,
                accent: crate::zsh::theme::color_scheme::AccentColor::Rainbow(
                    NamedColor::FullColor((0, 255, 255)),
                ),
                accent_which: AccentWhich::ForeGround,
            },
            connection: PromptConnection::Line,
            left_segment_separators: PromptSegmentSeparators {
                start_separator: PromptSeparation::Round,
                mid_separator: PromptSeparation::Slash,
                end_separator: PromptSeparation::Sharp,
                edge_cap: true,
                bold_separation: true,
            },
            right_segment_separators: PromptSegmentSeparators {
                start_separator: PromptSeparation::Sharp,
                mid_separator: PromptSeparation::BackSlash,
                end_separator: PromptSeparation::Round,
                edge_cap: true,
                bold_separation: true,
            },
            accent_which: AccentWhich::ForeGround,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PromptContent {
    /// 固定文字列を表示
    Literal {
        value: String,
        #[serde(with = "super::named_color_serde_option", default)]
        fg: Option<NamedColor>,
        #[serde(with = "super::named_color_serde_option", default)]
        bg: Option<NamedColor>,
    },
    /// プロセス内で直接実行（現在のバイナリ内で完結）
    BuildIn { command: zsh_prompts::Commands },
    /// 外部コマンドを実行
    Shell {
        cmd: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        envs: HashMap<String, String>,
        #[serde(with = "super::named_color_serde_option", default)]
        fg: Option<NamedColor>,
        #[serde(with = "super::named_color_serde_option", default)]
        bg: Option<NamedColor>,
    },
}

impl PromptContent {
    pub async fn content(&self) -> Vec<ZshSequence> {
        match self {
            // 1. Literal の処理
            Self::Literal { value, fg, bg } => {
                let mut seqs = Vec::new();
                if let Some(c) = bg {
                    seqs.push(ZshSequence::BackgroundColor(*c));
                }
                if let Some(c) = fg {
                    seqs.push(ZshSequence::ForegroundColor(*c));
                }
                seqs.push(ZshSequence::Literal(value.clone()));
                if fg.is_some() {
                    seqs.push(ZshSequence::ForegroundColorEnd);
                }
                if bg.is_some() {
                    seqs.push(ZshSequence::BackgroundColorEnd);
                }
                seqs
            }

            // 3. Build-in の処理 (現在のプロセスで直接実行)
            Self::BuildIn { command } => {
                let segments = command.exec();
                Self::convert_segments_to_sequences(segments)
            }

            // 4. Shell の処理
            Self::Shell {
                cmd,
                args,
                envs,
                fg,
                bg,
            } => {
                let mut command = Command::new(cmd);

                let expanded_args: Vec<String> = args
                    .iter()
                    .map(|arg| {
                        shellexpand::env(arg)
                            .unwrap_or(Cow::Borrowed(arg))
                            .to_string()
                    })
                    .collect();

                command.args(&expanded_args);
                if let Ok(current_dir) = std::env::current_dir() {
                    command.current_dir(current_dir);
                }
                for (key, value) in envs {
                    command.env(key, value);
                }

                if let Ok(output) = command.output().await
                    && output.status.success()
                {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !stdout.is_empty() {
                        let mut seqs = Vec::new();
                        if let Some(c) = bg {
                            seqs.push(ZshSequence::BackgroundColor(*c));
                        }
                        if let Some(c) = fg {
                            seqs.push(ZshSequence::ForegroundColor(*c));
                        }
                        seqs.push(ZshSequence::Literal(stdout));
                        if fg.is_some() {
                            seqs.push(ZshSequence::ForegroundColorEnd);
                        }
                        if bg.is_some() {
                            seqs.push(ZshSequence::BackgroundColorEnd);
                        }
                        return seqs;
                    }
                }
                Vec::new()
            }
        }
    }

    /// PromptSegment のリストを ZshSequence のリストに変換する補助関数
    fn convert_segments_to_sequences(
        segments: Vec<zsh_prompts::PromptSegment>,
    ) -> Vec<ZshSequence> {
        let mut result = Vec::new();
        let len = segments.len();

        for (i, segment) in segments.into_iter().enumerate() {
            if let Some(fg) = segment.color {
                result.push(ZshSequence::ForegroundColor(fg.to_named_color()));
            }
            result.push(ZshSequence::Literal(segment.content));
            if segment.color.is_some() {
                result.push(ZshSequence::ForegroundColorEnd);
            }
            // セグメント間のスペース
            if i < len - 1 {
                result.push(ZshSequence::Literal(" ".to_string()));
            }
        }
        result
    }
}

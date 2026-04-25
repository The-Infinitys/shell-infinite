mod color_named_color;
pub mod color_scheme;
pub mod config_ui;
pub mod gradient;
pub mod manager;
pub mod named_color_serde; // 既存のファイルをそのまま使用
pub mod named_color_serde_option; // 新しく追加
pub mod prompt_theme;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;

use crate::{
    args::ThemeCommands,
    zsh::theme::prompt_theme::{PromptContent, PromptContents, PromptTheme},
};

pub fn set(theme: ThemeCommands) {
    let theme = match theme {
        ThemeCommands::Default => PromptTheme::default(),
        ThemeCommands::Infinite => PromptTheme::infinite(),
    };
    let _ = manager::save_theme(&theme);
}
pub async fn main() {
    let mut current_theme = manager::load_theme();
    loop {
        println!("\n--- Zsh Infinite Theme Configuration ---");

        // メインメニューのオプションを動的に生成
        let mut options = vec![
            "Add new prompt line".to_string(),
            "Remove last prompt line".to_string(),
        ];
        for (i, _) in current_theme.prompt_contents_list.iter().enumerate() {
            options.push(format!("Configure Prompt Line {}", i));
        }
        options.push("Save and Exit".to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Main Menu")
            .items(&options)
            .clear(true)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => {
                // Add new prompt line
                current_theme
                    .prompt_contents_list
                    .push(PromptContents::default());
                println!("New prompt line added.");
            }
            Some(1) => {
                // Remove last prompt line
                if current_theme.prompt_contents_list.pop().is_some() {
                    println!("Last prompt line removed.");
                } else {
                    println!("No prompt lines to remove.");
                }
            }
            Some(s) if s >= 2 && s < options.len() - 1 => {
                // Configure Prompt Line
                let line_index = s - 2;
                if let Some(prompt_contents) =
                    current_theme.prompt_contents_list.get_mut(line_index)
                {
                    configure_prompt_line(prompt_contents).await;
                } else {
                    eprintln!("Invalid prompt line index selected.");
                }
            }
            Some(s) if s == options.len() - 1 => {
                // Save and Exit
                let _ = manager::save_theme(&current_theme);
                break;
            }
            _ => break, // Exit loop on cancel or error
        }
    }
}

async fn configure_prompt_line(prompt_contents: &mut PromptContents) {
    loop {
        println!("\n--- Configure Prompt Line ---");
        let options = [
            "Configure Colors",
            "Configure Connection",
            "Configure Separators",
            "Configure Left Prompt Content",  // 新しいオプション
            "Configure Right Prompt Content", // 新しいオプション
            "Back to Main Menu",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Prompt Line Configuration")
            .items(options)
            .clear(true)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => config_ui::configure_colors(prompt_contents),
            Some(1) => config_ui::configure_connection(prompt_contents),
            Some(2) => config_ui::configure_separation(prompt_contents),
            Some(3) => configure_prompt_content_list(&mut prompt_contents.left, "Left"),
            Some(4) => configure_prompt_content_list(&mut prompt_contents.right, "Right"),
            Some(5) | None => break,
            _ => unreachable!(),
        }
    }
}

fn configure_prompt_content_list(contents: &mut [PromptContent], side: &str) {
    loop {
        println!("\n--- Configure {} Prompt Contents ---", side);

        // 列挙型のバリアントに基づいたラベルの動的生成
        let mut options: Vec<String> = contents
            .iter()
            .enumerate()
            .map(|(i, pc)| {
                let label = match pc {
                    PromptContent::Literal { value, .. } => {
                        format!("Literal: \"{}\"", value)
                    }
                    PromptContent::BuildIn { command } => {
                        format!("Built-in: {:?}", command)
                    }
                    PromptContent::Shell { cmd, .. } => {
                        format!("External Shell: {}", cmd)
                    }
                };
                format!("{}: {}", i, label)
            })
            .collect();

        options.push("Back to Prompt Line Menu".to_string());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Select {} content to configure", side))
            .items(&options)
            .clear(true)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(s) if s == options.len() - 1 => break, // Back to Prompt Line Menu
            Some(selection) => {
                // 選択されたコンテンツの編集
                if let Some(prompt_content) = contents.get_mut(selection) {
                    // 色の設定メニュー（既存のUI関数を呼び出し）
                    config_ui::configure_prompt_content_colors(prompt_content);
                } else {
                    eprintln!("Invalid selection.");
                }
            }
            None => break,
        }
    }
}

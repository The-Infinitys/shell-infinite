use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use std::fmt;
use zsh_seq::NamedColor;

use super::gradient::create_default_rainbow_gradient;
use super::named_color_serde;
use super::prompt_theme::{PromptContent, PromptContents, PromptSegmentSeparators}; // PromptSegmentSeparatorsとPromptContentをインポート
use crate::zsh::prompt::{PromptConnection, PromptSeparation}; // crateルートからのパス

// DisplayNamedColor
struct DisplayNamedColor<'a>(Option<&'a NamedColor>); // Option<&'a NamedColor>を受け取るように変更
impl<'a> fmt::Display for DisplayNamedColor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(color) = self.0 {
            // named_color_serde内のデシリアライザが期待する形式に合わせる
            let s = match color {
                NamedColor::Code256(c) => format!("Code256({})", c),
                NamedColor::FullColor((r, g, b)) => format!("#{:02X}{:02X}{:02X}", r, g, b),
                _ => format!("{:?}", color),
            };
            write!(f, "{}", s)
        } else {
            write!(f, "None") // Noneの場合は"None"と表示
        }
    }
}

pub fn prompt_for_named_color(
    prompt_text: &str,
    default_color: Option<&NamedColor>,
) -> Option<NamedColor> {
    let default_str = DisplayNamedColor(default_color).to_string();
    let input = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt_text)
        .default(default_str.clone())
        .interact_text()
        .unwrap_or_else(|_| default_str.clone());

    if input.eq_ignore_ascii_case("None") || input.is_empty() {
        return None;
    }

    if let Ok(color) = named_color_serde::deserialize_from_str(&input) {
        Some(color)
    } else {
        println!("Invalid color format. Keeping default or setting to None if default was None.");
        default_color.cloned() // 不正な入力の場合はデフォルト値を返す
    }
}

// 新しい関数: フルカラーのRGB値をプロンプトで取得
pub fn prompt_for_rgb_color(prompt_text: &str, default_rgb: (u8, u8, u8)) -> (u8, u8, u8) {
    let default_hex = format!(
        "#{:02X}{:02X}{:02X}",
        default_rgb.0, default_rgb.1, default_rgb.2
    );
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt_text)
        .default(default_hex.clone())
        .interact_text()
        .map(|s| {
            if s.starts_with('#')
                && s.len() == 7
                && let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&s[1..3], 16),
                    u8::from_str_radix(&s[3..5], 16),
                    u8::from_str_radix(&s[5..7], 16),
                )
            {
                return (r, g, b);
            }
            default_rgb
        })
        .unwrap_or(default_rgb)
}

pub fn configure_colors(prompt_contents: &mut PromptContents) {
    println!("\n--- Configure Colors ---");

    prompt_contents.color.bg =
        prompt_for_named_color("Background color", Some(&prompt_contents.color.bg))
            .unwrap_or(NamedColor::Black);
    prompt_contents.color.fg =
        prompt_for_named_color("Foreground color", Some(&prompt_contents.color.fg))
            .unwrap_or(NamedColor::White);
    prompt_contents.color.pc =
        prompt_for_named_color("Primary color (pc)", Some(&prompt_contents.color.pc))
            .unwrap_or(NamedColor::Cyan);
    prompt_contents.color.sc =
        prompt_for_named_color("Secondary color (sc)", Some(&prompt_contents.color.sc))
            .unwrap_or(NamedColor::LightBlack);

    let options = [
        "Single Color",
        "Rainbow",
        "Default Rainbow Gradient",
        "Custom Gradient",
    ];
    let default_selection = match &prompt_contents.color.accent {
        super::color_scheme::AccentColor::Single(_) => 0,
        super::color_scheme::AccentColor::Rainbow(_) => 1,
        super::color_scheme::AccentColor::Gradient(_) => 3, // Custom Gradient に対応
    };

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose separation color type")
        .items(options)
        .default(default_selection)
        .clear(true)
        .interact_opt()
        .unwrap_or(None);

    if let Some(selection) = selection {
        prompt_contents.color.accent = match selection {
            0 => super::color_scheme::AccentColor::Single(
                prompt_for_named_color("Color", Some(&NamedColor::LightBlack))
                    .unwrap_or(NamedColor::LightBlack),
            ),
            1 => {
                let color = prompt_for_named_color(
                    "Rainbow Start Color (Hex)",
                    Some(&NamedColor::FullColor((255, 0, 0))),
                )
                .unwrap_or(NamedColor::FullColor((255, 0, 0)));
                super::color_scheme::AccentColor::Rainbow(color)
            }
            2 => {
                // Default Rainbow Gradient
                super::color_scheme::AccentColor::Gradient(create_default_rainbow_gradient())
            }
            3 => {
                // Custom Gradient (Existing 2-point gradient)
                let c1_rgb = prompt_for_rgb_color("Gradient Start Color (Hex)", (0, 255, 255)); // Cyan
                let c2_rgb = prompt_for_rgb_color("Gradient End Color (Hex)", (0, 0, 255)); // Blue
                super::color_scheme::AccentColor::Gradient(vec![(c1_rgb, 0.0), (c2_rgb, 1.0)])
            }
            _ => unreachable!(),
        };
    }
}

pub fn configure_prompt_content_colors(prompt_content: &mut PromptContent) {
    println!("\n--- Configure Prompt Content Colors ---");

    match prompt_content {
        PromptContent::Literal { fg, bg, .. } | PromptContent::Shell { fg, bg, .. } => {
            // Foreground の設定
            *fg = prompt_for_named_color("Foreground color (enter 'None' to clear)", fg.as_ref());

            // Background の設定
            *bg = prompt_for_named_color("Background color (enter 'None' to clear)", bg.as_ref());
        }
        PromptContent::BuildIn { .. } => {
            println!("Note: Colors for Daemon/BuildIn are managed by their respective commands.");
            println!("(They use the segment's internal color settings.)");
            // 必要であれば、ここで「強制的に上書きする色」を設定するロジックを
            // 構造体側に追加することも検討してください。
        }
    }
}

pub fn configure_connection(prompt_contents: &mut PromptContents) {
    println!("\n--- Configure Connection ---");
    let options = [
        PromptConnection::None,
        PromptConnection::Line,
        PromptConnection::Double,
        PromptConnection::Bold,
        PromptConnection::Dashed,
        PromptConnection::Dotted,
        PromptConnection::Dot,
        PromptConnection::Bullet,
        PromptConnection::Wave,
        PromptConnection::ZigZag,
        PromptConnection::Bar,
        PromptConnection::Gradient,
    ];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose style")
        .items(
            options
                .iter()
                .map(|o| format!("{:?}", o))
                .collect::<Vec<_>>(),
        )
        .default(
            options
                .iter()
                .position(|&p| p == prompt_contents.connection)
                .unwrap_or(0),
        )
        .clear(true)
        .interact_opt()
        .unwrap_or(None);
    if let Some(selection) = selection {
        prompt_contents.connection = options[selection];
    }
}

// PromptSeparationの選択UIをヘルパー関数として抽出
fn select_prompt_separation_style(current_style: &PromptSeparation) -> PromptSeparation {
    let options = [
        PromptSeparation::Block,
        PromptSeparation::Sharp,
        PromptSeparation::Slash,
        PromptSeparation::BackSlash,
        PromptSeparation::Round,
        PromptSeparation::Blur,
        PromptSeparation::Flame,
        PromptSeparation::Pixel,
        PromptSeparation::Wave,
        PromptSeparation::Lego,
    ];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose style")
        .items(
            options
                .iter()
                .map(|o| format!("{:?}", o))
                .collect::<Vec<_>>(),
        )
        .default(
            options
                .iter()
                .position(|&p| p == *current_style)
                .unwrap_or(0),
        )
        .clear(true)
        .interact_opt()
        .unwrap_or(None);
    if let Some(selection) = selection {
        options[selection]
    } else {
        *current_style
    }
}

// PromptSegmentSeparatorsを設定する新しい関数
pub fn configure_segment_separators(segment_separators: &mut PromptSegmentSeparators) {
    loop {
        println!("\n--- Configure Segment Separators ---");
        let options = [
            "Start Separator",
            "Mid Separator",
            "End Separator",
            "Back to Separation Menu",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Segment to Configure")
            .items(options)
            .clear(true)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => {
                segment_separators.start_separator =
                    select_prompt_separation_style(&segment_separators.start_separator);
            }
            Some(1) => {
                segment_separators.mid_separator =
                    select_prompt_separation_style(&segment_separators.mid_separator);
            }
            Some(2) => {
                segment_separators.end_separator =
                    select_prompt_separation_style(&segment_separators.end_separator);
            }
            Some(3) | None => break,
            _ => unreachable!(),
        }
    }
}

pub fn configure_separation(prompt_contents: &mut PromptContents) {
    loop {
        println!("\n--- Configure Separators ---");
        let options = [
            "Left Segment Separators",
            "Right Segment Separators",
            "Back to Prompt Line Menu",
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select Side to Configure Separators")
            .items(options)
            .clear(true)
            .interact_opt()
            .unwrap_or(None);

        match selection {
            Some(0) => configure_segment_separators(&mut prompt_contents.left_segment_separators),
            Some(1) => configure_segment_separators(&mut prompt_contents.right_segment_separators),
            Some(2) | None => break,
            _ => unreachable!(),
        }
    }
}

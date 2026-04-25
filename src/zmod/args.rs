use clap::{Parser, Subcommand};

use crate::zmod::ZshInfinite;

#[derive(Parser)]
#[command(name = "zsh-infinite-internal")]
pub struct ZmodArgs {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Get { key: String },
    Store { key: String, value: String },
    Precmd,
    LineFinish,
    Interrupt,
    Cleanup,
}

impl ZmodArgs {
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // &self.command とすることで所有権の移動（Partial Move）を防ぐ
        match &self.command {
            Commands::Get { key } => {
                if let Some(value) = self.fetch_internal_state(key) {
                    println!("{}", value);
                }
            }
            Commands::Store { key, value } => {
                self.save_internal_state(key, value)?;
            }
            Commands::Precmd => ZshInfinite::with_instance(|zsh_infinite| zsh_infinite.precmd())?,
            Commands::LineFinish => {
                ZshInfinite::with_instance(|zsh_infinite| zsh_infinite.line_finish())?
            }
            Commands::Interrupt => {
                ZshInfinite::with_instance(|zsh_infinite| zsh_infinite.interrupt())?
            }
            Commands::Cleanup => {
                self.perform_cleanup()?;
            }
        }
        Ok(())
    }

    // 未使用変数警告を避けるため引数にアンダースコアを付与
    fn fetch_internal_state(&self, _key: &str) -> Option<String> {
        None
    }

    fn save_internal_state(
        &self,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn perform_cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

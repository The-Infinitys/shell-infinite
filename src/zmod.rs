use clap::Parser;
use tokio::runtime::Runtime;
use zsh_system::{Features, ZshModule, ZshParameter, ZshResult, eval, export_module};

mod args;
use crate::{args::PromptType, zsh};
use args::ZmodArgs;
#[derive(Default)]
struct ZshInfinite {
    rt: Option<Runtime>,
    old_prompt: String,
    old_rprompt: String,
}

impl ZshInfinite {
    pub fn precmd(&mut self) -> ZshResult {
        if self.rt.is_none() {
            self.rt = Some(Runtime::new().unwrap());
        }
        let rt = self.rt.as_ref().unwrap();

        let left_prompt = rt.block_on(async { zsh::build_prompt(&PromptType::Left).await.build() });
        let right_prompt =
            rt.block_on(async { zsh::build_prompt(&PromptType::Right).await.build() });
        ZshParameter::set_str("PROMPT", &left_prompt)?;
        ZshParameter::set_str("RPROMPT", &right_prompt)?;
        Ok(())
    }
    pub fn line_finish(&mut self) -> ZshResult {
        if self.rt.is_none() {
            self.rt = Some(Runtime::new().unwrap());
        }
        let rt = self.rt.as_ref().unwrap();

        let exit_code = ZshParameter::get_int("?") as i32;
        let transient_prompt = rt.block_on(async {
            zsh::build_prompt(&PromptType::Transient {
                exit_code: Some(exit_code),
            })
            .await
            .build()
        });
        ZshParameter::set_str("PROMPT", &transient_prompt)?;
        ZshParameter::set_str("RPROMPT", "")?;
        eval("zle reset-prompt");
        Ok(())
    }
    pub fn interrupt(&mut self) -> ZshResult {
        if self.rt.is_none() {
            self.rt = Some(Runtime::new().unwrap());
        }
        let rt = self.rt.as_ref().unwrap();

        let transient_prompt = rt.block_on(async {
            zsh::build_prompt(&PromptType::Transient {
                exit_code: Some(130),
            })
            .await
            .build()
        });
        ZshParameter::set_str("PROMPT", &transient_prompt)?;
        ZshParameter::set_str("RPROMPT", "")?;
        eval("zle reset-prompt");
        eval("zle -R");
        Ok(())
    }
}
impl ZshModule for ZshInfinite {
    fn setup(&mut self) -> ZshResult {
        Ok(())
    }

    fn boot(&mut self) -> ZshResult {
        self.rt = Some(Runtime::new().unwrap());
        self.old_prompt = ZshParameter::get_str("PROMPT").unwrap_or_default();
        self.old_rprompt = ZshParameter::get_str("RPROMPT").unwrap_or_default();
        zsh_system::eval(include_str!("assets/scripts/zmod/boot.sh"));
        Ok(())
    }

    fn features(&self) -> Features {
        Features::new().add_builtin("__zsh_infinite_internal", |name, args| {
            let args = std::iter::once(name).chain(args.iter().copied());
            match ZmodArgs::try_parse_from(args) {
                Ok(zmod_args) => match zmod_args.run() {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("{}", e);
                        1
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    2
                }
            }
        })
    }

    fn cleanup(&mut self) -> ZshResult {
        zsh_system::eval(include_str!("assets/scripts/zmod/cleanup.sh"));
        self.rt = None;
        ZshParameter::set_str("PROMPT", &self.old_prompt)?;
        ZshParameter::set_str("RPROMPT", &self.old_rprompt)?;
        eprintln!("[ZshInfinite] Module unloaded successfully.");
        Ok(())
    }
    fn finish(&mut self) -> ZshResult {
        Ok(())
    }
}

// マクロでエクスポート
export_module!(ZshInfinite);

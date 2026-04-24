use directories::ProjectDirs;
use std::{env, io, path::PathBuf};

pub const QUALIFIER: &str = "org";
pub const ORGANIZATION: &str = "infinite";
pub const APPLICATION: &str = "zsh-infinite";
pub const ZSH_THEME_FILE_NAME: &str = "infinite.zsh-theme";
pub const ZSH_RC_SNIPPET_FILE_NAME: &str = "infinite_zshrc_snippet"; // To be sourced by user's .zshrc

#[derive(Debug)]
pub struct InstallPaths {
    pub bin_dir: PathBuf,
    pub theme_file_path: PathBuf,
    pub zshrc_snippet_path: PathBuf,
    pub is_oh_my_zsh_install: bool,
    pub sheldon_config_path: Option<PathBuf>,
}

pub fn is_oh_my_zsh_installed() -> bool {
    if let Ok(zsh_path) = env::var("ZSH") {
        let oh_my_zsh_path = PathBuf::from(zsh_path).join("oh-my-zsh.sh");
        oh_my_zsh_path.exists()
    } else {
        false
    }
}

pub fn get_sheldon_config_path() -> Option<PathBuf> {
    if let Ok(config_file) = env::var("SHELDON_CONFIG_FILE") {
        let path = PathBuf::from(config_file);
        if path.exists() {
            return Some(path);
        }
    }

    let home_dir = env::var("HOME").ok()?;
    let default_path = PathBuf::from(home_dir).join(".config/sheldon/plugins.toml");
    if default_path.exists() {
        Some(default_path)
    } else {
        None
    }
}

pub fn get_oh_my_zsh_root() -> Option<PathBuf> {
    env::var("ZSH").ok().map(PathBuf::from)
}

pub fn get_oh_my_zsh_custom_theme_dir() -> Option<PathBuf> {
    if let Ok(zsh_custom) = env::var("ZSH_CUSTOM") {
        Some(PathBuf::from(zsh_custom).join("themes"))
    } else {
        get_oh_my_zsh_root().map(|zsh_root| zsh_root.join("custom").join("themes"))
    }
}

pub fn get_install_paths() -> Result<InstallPaths, io::Error> {
    let is_oh_my_zsh_install = is_oh_my_zsh_installed();
    let sheldon_config_path = get_sheldon_config_path();

    if is_oh_my_zsh_install {
        let home_dir = env::var("HOME").map_err(io::Error::other)?;
        let bin_dir = PathBuf::from(&home_dir).join(".local").join("bin"); // Oh My Zsh環境でもバイナリはユーザーローカルに置く

        let theme_file_path = get_oh_my_zsh_custom_theme_dir()
            .ok_or_else(|| {
                io::Error::other("Could not determine Oh My Zsh custom theme directory.")
            })?
            .join(ZSH_THEME_FILE_NAME);

        let zshrc_snippet_path = PathBuf::from(&home_dir)
            .join(".config")
            .join(APPLICATION)
            .join(ZSH_RC_SNIPPET_FILE_NAME);

        Ok(InstallPaths {
            bin_dir,
            theme_file_path,
            zshrc_snippet_path,
            is_oh_my_zsh_install,
            sheldon_config_path,
        })
    } else if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        let bin_dir = proj_dirs.data_local_dir().join("bin");
        let theme_file_path = proj_dirs.config_dir().join(ZSH_THEME_FILE_NAME);
        let zshrc_snippet_path = proj_dirs.config_dir().join(ZSH_RC_SNIPPET_FILE_NAME);

        Ok(InstallPaths {
            bin_dir,
            theme_file_path,
            zshrc_snippet_path,
            is_oh_my_zsh_install,
            sheldon_config_path,
        })
    } else {
        Err(io::Error::other(
            "Could not determine project directories for installation.",
        ))
    }
}

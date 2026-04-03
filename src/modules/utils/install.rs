#[cfg(zsh_lib_found)]
use super::paths;
#[cfg(all(unix, zsh_lib_found))]
use std::os::unix::fs::PermissionsExt;
#[cfg(zsh_lib_found)]
use std::{env, fs, path::PathBuf};

#[cfg(zsh_lib_found)]
pub fn install() {
    const LIB_DATA: &[u8] = include_bytes!(env!("ZSH_LIB_PATH"));
    let install_paths = match paths::get_install_paths() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("Error determining installation paths: {}", e);
            return;
        }
    };
    let home_dir = match env::var("HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            eprintln!("Error: HOME environment variable not set.");
            return;
        }
    };
    let lib_dir = home_dir.join(".local/lib");
    if let Err(e) = fs::create_dir_all(&lib_dir) {
        eprintln!("Error creating library directory {:?}: {}", lib_dir, e);
        return;
    }

    // OSに応じた拡張子の決定
    let lib_file_name = if cfg!(target_os = "macos") {
        "libzsh_infinite.dylib"
    } else {
        "libzsh_infinite.so"
    };
    let lib_dest_path = lib_dir.join(lib_file_name);

    match fs::write(&lib_dest_path, LIB_DATA) {
        Ok(_) => {
            println!("Shared library installed to: {:?}", lib_dest_path);
            // 実行権限の付与（必要に応じて）
            #[cfg(unix)]
            let _ = fs::set_permissions(&lib_dest_path, fs::Permissions::from_mode(0o755));
        }
        Err(e) => {
            eprintln!(
                "Error installing shared library to {:?}: {}",
                lib_dest_path, e
            );
            return;
        }
    }
    // 1. Create necessary directories for bin
    if let Err(e) = fs::create_dir_all(&install_paths.bin_dir) {
        eprintln!(
            "Error creating binary directory {:?}: {}",
            install_paths.bin_dir, e
        );
        return;
    }

    // 2. Copy the current executable
    let current_exe_path = match env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error getting current executable path: {}", e);
            return;
        }
    };
    let target_exe_name = current_exe_path
        .file_name()
        .expect("Failed to get file name");
    let target_exe_path = install_paths.bin_dir.join(target_exe_name);

    if current_exe_path == target_exe_path {
        println!(
            "Executable already exists at target path: {:?}",
            target_exe_path
        );
    } else {
        match fs::copy(&current_exe_path, &target_exe_path) {
            Ok(_) => println!("Executable copied to: {:?}", target_exe_path),
            Err(e) => {
                eprintln!(
                    "Error copying executable from {:?} to {:?}: {}",
                    current_exe_path, target_exe_path, e
                );
                return;
            }
        }
    }

    // 3. Generate infinite.zsh-theme
    let theme_template = include_str!("../../assets/scripts/infinite.zsh-theme").to_string();
    let theme_content = theme_template.replace(
        "{{RUN_DIR}}",
        &target_exe_path
            .parent()
            .expect("Binary path should have a parent directory")
            .to_string_lossy(),
    );

    if let Err(e) = fs::create_dir_all(
        install_paths
            .theme_file_path
            .parent()
            .expect("Theme file path should have a parent directory"),
    ) {
        eprintln!(
            "Error creating theme directory {:?}: {}",
            install_paths.theme_file_path.parent().unwrap(),
            e
        );
        return;
    }

    match fs::write(&install_paths.theme_file_path, theme_content) {
        Ok(_) => println!("Theme file created at: {:?}", install_paths.theme_file_path),
        Err(e) => {
            eprintln!(
                "Error writing theme file to {:?}: {}",
                install_paths.theme_file_path, e
            );
            return;
        }
    }

    // 4. Modify user's ~/.zshrc
    let user_zshrc_path = home_dir.join(".zshrc");

    if !user_zshrc_path.exists() {
        println!(
            "User's ~/.zshrc not found at {:?}. Please create it.",
            user_zshrc_path
        );
        return;
    }

    match fs::read_to_string(&user_zshrc_path) {
        Ok(zshrc_content) => {
            let mut _modified = false;
            let mut new_zshrc_content_lines: Vec<String> = Vec::new();

            if install_paths.is_oh_my_zsh_install {
                // Oh My Zsh installation
                let theme_name = install_paths
                    .theme_file_path
                    .file_stem()
                    .expect("Theme file name not found")
                    .to_string_lossy();
                let theme_setting_line = format!("ZSH_THEME=\"{}\"", theme_name);
                let source_oh_my_zsh_line = "source $ZSH/oh-my-zsh.sh";

                let mut theme_found = false;
                let mut source_omz_found = false;

                for line in zshrc_content.lines() {
                    let trimmed_line = line.trim();

                    if (trimmed_line.starts_with("ZSH_THEME=")
                        || trimmed_line.starts_with("export ZSH_THEME="))
                        && !theme_found
                    {
                        new_zshrc_content_lines.push(theme_setting_line.clone());
                        theme_found = true;
                        _modified = true;
                    } else if trimmed_line.contains(source_oh_my_zsh_line) && !source_omz_found {
                        if !theme_found {
                            new_zshrc_content_lines.push(theme_setting_line.clone());
                            theme_found = true;
                            _modified = true;
                        }
                        new_zshrc_content_lines.push(line.to_string());
                        source_omz_found = true;
                    } else {
                        new_zshrc_content_lines.push(line.to_string());
                    }
                }

                if !theme_found {
                    if !new_zshrc_content_lines.is_empty() {
                        new_zshrc_content_lines.push(String::new());
                    }
                    new_zshrc_content_lines.push(theme_setting_line);
                    if !source_omz_found {
                        new_zshrc_content_lines.push(source_oh_my_zsh_line.to_string());
                    }
                    _modified = true;
                }

                if _modified {
                    let final_zshrc_content = new_zshrc_content_lines.join("\n");
                    match fs::write(&user_zshrc_path, final_zshrc_content.as_bytes()) {
                        Ok(_) => println!("~/.zshrc updated for Oh My Zsh theme."),
                        Err(e) => eprintln!("Error writing to ~/.zshrc: {}", e),
                    }
                } else {
                    println!(
                        "~/.zshrc already configured for Oh My Zsh theme. Skipping modification."
                    );
                }
            } else {
                // Standalone installation
                let source_line = format!(
                    "source \"{}\"",
                    install_paths.zshrc_snippet_path.to_string_lossy()
                );
                let installer_comment_start = "# Added by zsh-infinite installer";

                let mut source_line_present = false;
                for line in zshrc_content.lines() {
                    if line.contains(&source_line) {
                        source_line_present = true;
                        break;
                    }
                }

                if !source_line_present {
                    let mut zshrc_lines: Vec<String> =
                        zshrc_content.lines().map(|s| s.to_string()).collect();
                    if !zshrc_lines.is_empty() {
                        zshrc_lines.push(String::new());
                    }
                    zshrc_lines.push(installer_comment_start.to_string());
                    zshrc_lines.push(source_line.to_string());

                    match fs::write(&user_zshrc_path, zshrc_lines.join("\n").as_bytes()) {
                        Ok(_) => println!("~/.zshrc updated for standalone theme."),
                        Err(e) => eprintln!("Error writing to ~/.zshrc: {}", e),
                    }
                    _modified = true;
                }

                // Create snippet file
                let zshrc_snippet_content = format!(
                    r#"
# Added by zsh-infinite installer
if [ -f "{}" ]; then
    source "{}"
fi
"#,
                    install_paths.theme_file_path.to_string_lossy(),
                    install_paths.theme_file_path.to_string_lossy()
                );
                if let Err(e) = fs::write(
                    &install_paths.zshrc_snippet_path,
                    zshrc_snippet_content.as_bytes(),
                ) {
                    eprintln!(
                        "Error writing zshrc snippet to {:?}: {}",
                        install_paths.zshrc_snippet_path, e
                    );
                    return;
                } else if !source_line_present {
                    println!(
                        "Zshrc snippet created at: {:?}",
                        install_paths.zshrc_snippet_path
                    );
                }
            }
        }
        Err(e) => eprintln!("Error reading ~/.zshrc: {}", e),
    }

    println!(
        "\nInstallation complete! Please restart your Zsh session or run 'source ~/.zshrc' to apply the changes."
    );
}

#[cfg(not(zsh_lib_found))]
pub fn install() {
    // ライブラリビルド時には何もしない
}

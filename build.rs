// プロジェクトルートの build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    // `zsh_lib_found` cfg をcargoに通知して、`unexpected_cfgs`警告を抑制する
    println!("cargo:rustc-check-cfg=cfg(zsh_lib_found)");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap(); // debug or release

    // ライブラリの出力先を決定
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let lib_name = "zsh_infinite";
    let (prefix, ext) = if target_os == "macos" {
        ("lib", "dylib")
    } else {
        ("lib", "so")
    };

    // 通常、同じワークスペースなら target/{profile}/ 配下に生成されます
    // ワークスペース構成に合わせてパスを調整してください
    let lib_path = PathBuf::from(&manifest_dir)
        .join("target")
        .join(&profile)
        .join(format!("{}{}.{}", prefix, lib_name, ext));

    // バイナリビルド時（＝ライブラリファイルが既に存在する場合）のみ、
    // 環境変数をセットし、cfgフラグを有効にする
    println!("cargo:rerun-if-changed={}", lib_path.display());
    if lib_path.exists() {
        println!("cargo:rustc-env=ZSH_LIB_PATH={}", lib_path.display());
        println!("cargo:rustc-cfg=zsh_lib_found");
    } else {
        println!(
            "cargo:warning=Shared library not found at {}. Install command will be restricted.",
            lib_path.display()
        );
    }
}

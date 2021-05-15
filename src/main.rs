use directories::ProjectDirs;

use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let cfg_dir = ProjectDirs::from("", "", "xtpost").unwrap().config_dir().to_path_buf();

    if !cfg_dir.exists() {
        create_dir_all(&cfg_dir).unwrap();
    }

    open_editor(&cfg_dir);
}

fn open_editor(cfg_dir: &PathBuf) {
    let editor = if cfg!(target_os = "windows") {
        "notepad"
    } else {
        "vi"
    };

    let mut cfg_path = cfg_dir.clone();
    cfg_path.push("xtpost.json");

    Command::new(editor)
        .args(&[cfg_path])
        .status()
        .expect("editor failed to start");
}

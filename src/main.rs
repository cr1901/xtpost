use directories::ProjectDirs;
use serde_json;

use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

mod cfg;

fn main() {
    let cfg_dir = ProjectDirs::from("", "", "xtpost").unwrap().config_dir().to_path_buf();
    let mut cfg_file = cfg_dir.clone();
    cfg_file.push("settings.json");

    if !cfg_dir.exists() {
        create_dir_all(&cfg_dir).unwrap();
    }

    if !cfg_file.exists() {
        let json_cfg = serde_json::to_string_pretty::<cfg::Config>(&Default::default()).unwrap();
        let mut file = File::create(&cfg_file).unwrap();

        file.write_all(&json_cfg.into_bytes()).unwrap();
    }

    open_editor(&cfg_file);
}

fn open_editor(cfg_file: &PathBuf) {
    let editor = if cfg!(target_os = "windows") {
        "notepad"
    } else {
        "vi"
    };

    Command::new(editor)
        .args(&[cfg_file])
        .status()
        .expect("editor failed to start");
}

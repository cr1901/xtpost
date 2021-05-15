use directories::ProjectDirs;
use serde::{Serialize, Deserialize};

use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::process::Command;

const XT_SERVER: &str = "http://reenigne.mooo.com:8088/cgi-bin/xtserver.exe";

#[derive(Serialize, Deserialize, Debug)]
pub enum SessionType {
    Reenigne
}

impl Default for SessionType {
    fn default() -> Self {
        SessionType::Reenigne
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    email: Option<String>,
    server: String,
    session_type: SessionType,
}

impl Default for Config {
    fn default() -> Self {
        let server = String::from(XT_SERVER);

        Config {
            email: None,
            server,
            session_type: Default::default()
        }
    }
}

pub fn write_cfg_if_doesnt_exist() {
    let cfg_dir = ProjectDirs::from("", "", "xtpost").unwrap().config_dir().to_path_buf();
    let mut cfg_file = cfg_dir.clone();
    cfg_file.push("settings.json");

    if !cfg_dir.exists() {
        create_dir_all(&cfg_dir).unwrap();
    }

    if !cfg_file.exists() {
        let json_cfg = serde_json::to_string_pretty::<Config>(&Default::default()).unwrap();
        let mut file = File::create(&cfg_file).unwrap();

        file.write_all(&json_cfg.into_bytes()).unwrap();
    }
}

pub fn open_editor() {
    let cfg_dir = ProjectDirs::from("", "", "xtpost").unwrap().config_dir().to_path_buf();
    let mut cfg_file = cfg_dir.clone();
    cfg_file.push("settings.json");

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

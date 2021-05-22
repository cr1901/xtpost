use directories::ProjectDirs;
use eyre::{self, Result};
use once_cell::sync::OnceCell;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};

use std::env;
use std::error;
use std::fmt;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

const XT_SERVER: &str = "http://reenigne.mooo.com:8088/cgi-bin/xtserver.exe";
static CONFIG_DIR: OnceCell<PathBuf> = OnceCell::new();
static DATA_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn config_dir_name() -> Result<&'static PathBuf> {
    CONFIG_DIR.get_or_try_init::<_, eyre::Report>(|| {
        Ok(ProjectDirs::from("", "", "xtpost")
            .ok_or(Error::ConfigDirNotFound)?
            .config_dir()
            .to_path_buf())
    })
}

pub fn data_dir_name() -> Result<&'static PathBuf> {
    DATA_DIR.get_or_try_init::<_, eyre::Report>(|| {
        Ok(ProjectDirs::from("", "", "xtpost")
            .ok_or(Error::DataLocalDirNotFound)?
            .data_local_dir()
            .to_path_buf())
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SessionType {
    Reenigne,
}

impl Default for SessionType {
    fn default() -> Self {
        SessionType::Reenigne
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub email: Option<String>,
    pub server: String,
    pub session_type: SessionType,
}

#[derive(Debug)]
pub enum Error {
    ConfigDirNotFound,
    DataLocalDirNotFound,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigDirNotFound => write!(f, "could not find configuration directory"),
            Error::DataLocalDirNotFound => write!(f, "could not find application data directory"),
        }
    }
}

impl error::Error for Error {}

impl Default for Config {
    fn default() -> Self {
        let server = String::from(XT_SERVER);

        Config {
            email: None,
            server,
            session_type: Default::default(),
        }
    }
}

pub fn write_cfg_if_doesnt_exist() -> Result<()> {
    let cfg_dir: &PathBuf = config_dir_name()?;

    let mut cfg_file = cfg_dir.clone();
    cfg_file.push("settings.json");

    if !cfg_dir.exists() {
        create_dir_all(&cfg_dir)?;
    }

    if !cfg_file.exists() {
        let json_cfg = serde_json::to_string_pretty::<Config>(&Default::default())?;
        let mut file = File::create(&cfg_file)?;

        file.write_all(&json_cfg.into_bytes())?;
    }

    Ok(())
}

pub fn make_data_dir_if_doesnt_exist() -> Result<()> {
    let data_dir: &PathBuf = data_dir_name()?;

    if !data_dir.exists() {
        create_dir_all(&data_dir)?;
    }

    Ok(())
}

pub fn open_editor() -> Result<()> {
    let cfg_dir: &PathBuf = config_dir_name()?;

    let mut cfg_file = cfg_dir.clone();
    cfg_file.push("settings.json");

    Command::new(env::var("EDITOR")
        .unwrap_or_else(|_| if cfg!(target_os = "windows") {
            "notepad"
        } else {
            "vi"
        }.to_string())).args(&[cfg_file]).status()?;

    Ok(())
}

pub fn read_cfg_string() -> Result<String> {
    let cfg_dir: &PathBuf = config_dir_name()?;

    let mut cfg_file = cfg_dir.clone();
    cfg_file.push("settings.json");

    let mut file = File::open(cfg_file)?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    Ok(buffer)
}

pub fn read_cfg() -> Result<Config> {
    Ok(serde_json::from_str(&read_cfg_string()?)?)
}

pub fn url_to_data_dir<T>(url: T) -> Result<PathBuf>
where
    T: IntoUrl,
{
    let data_dir: &PathBuf = data_dir_name()?;
    let parsed_url = url.into_url()?;
    let filename = parsed_url
        .path_segments()
        .ok_or(eyre::eyre!("could not obtain path segments"))?
        .next_back()
        .ok_or(eyre::eyre!(
            "URL claims to have path, but last path segment was not found"
        ))?;

    let mut full_path = data_dir.clone();
    full_path.push(filename);

    Ok(full_path)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url_to_data_dir() {
        let data_dir: &PathBuf = data_dir_name().unwrap();

        let mut data_file = data_dir.clone();
        data_file.push("d5JESctuMKW88L-e.png");

        assert_eq!(
            data_file,
            url_to_data_dir("http://reenigne.mooo.com:8088/d5JESctuMKW88L-e.png").unwrap()
        );
    }
}

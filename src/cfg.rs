use serde::{Serialize, Deserialize};

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

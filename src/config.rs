use crate::prelude::*;
use std::fs;

pub static mut CONFIG: Option<Config> = None;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub db: String,
}

pub fn load_config() -> Config {
    let toml_config = fs::read_to_string("config.toml").unwrap();
    let config = toml::from_str(&toml_config);
    config.unwrap()
}

pub fn get_config() -> &'static Config {
    unsafe { CONFIG.as_ref().unwrap() }
}
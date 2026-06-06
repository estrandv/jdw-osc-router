use serde::Deserialize;
use std::net::SocketAddrV4;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind_address: String,
    pub bind_port: u16,
    pub buffer_size: usize,
}

impl Config {
    pub fn bind_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::from_str(&format!("{}:{}", self.bind_address, self.bind_port))
            .expect("Invalid bind address in config")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            bind_port: 13339,
            buffer_size: 333000,
        }
    }
}

pub fn load(path: &str) -> Config {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok())
        .unwrap_or_else(|| {
            let cfg = Config::default();
            eprintln!("No config at '{}', using defaults", path);
            cfg
        })
}

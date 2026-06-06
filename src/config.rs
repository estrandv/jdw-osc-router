use serde::Deserialize;
use std::net::SocketAddrV4;
use std::path::Path;
use std::str::FromStr;
use toml::Value as TomlValue;

static APP_NAME: &str = "router";

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

fn central_config_path() -> String {
    if let Ok(path) = std::env::var("JDW_CONFIG") {
        if Path::new(&path).exists() {
            return path;
        }
    }
    let home = std::env::var("HOME").ok();
    if let Some(home) = home {
        let xdg = Path::new(&home).join(".config").join("jdw.toml");
        if xdg.exists() {
            return xdg.to_string_lossy().to_string();
        }
    }
    eprintln!("Error: Central config not found at ~/.config/jdw.toml");
    eprintln!("       Set $JDW_CONFIG to a custom path, or create the file.");
    std::process::exit(1);
}

fn load_central_section() -> Option<TomlValue> {
    let path = central_config_path();
    let contents = std::fs::read_to_string(&path).ok()?;
    let root: TomlValue = contents.parse().ok()?;
    root.get(APP_NAME).cloned()
}

fn merge_config(base: &mut Config, overlay: &TomlValue) {
    if let Some(v) = overlay.get("bind_address").and_then(|v| v.as_str()) {
        base.bind_address = v.to_string();
    }
    if let Some(v) = overlay.get("bind_port").and_then(|v| v.as_integer()) {
        base.bind_port = v as u16;
    }
    if let Some(v) = overlay.get("buffer_size").and_then(|v| v.as_integer()) {
        base.buffer_size = v as usize;
    }
}

pub fn load(config_path: &str) -> Config {
    let mut cfg = Config::default();

    // Layer 1: central config (~/.config/jdw.toml)
    if let Some(central) = load_central_section() {
        merge_config(&mut cfg, &central);
    }

    // Layer 2: per-app config.toml (overrides central)
    if let Ok(contents) = std::fs::read_to_string(config_path) {
        if let Ok(local) = toml::from_str::<TomlValue>(&contents) {
            merge_config(&mut cfg, &local);
        }
    } else {
        eprintln!("Warning: Config file '{}' not found. Using defaults.", config_path);
    }

    cfg
}

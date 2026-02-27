use serde::Deserialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,

    #[serde(default)]
    pub proxies: HashMap<String, ProxieConfig>,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct RemoteProxieConfig {
    pub target: String,

    #[serde(default)]
    pub upstream_headers: HashMap<String, String>,

    #[serde(default)]
    pub downstream_headers: HashMap<String, String>,

    #[serde(default)]
    pub rewrite: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct LocalProxieConfig {
    pub target: String,

    #[serde(default)]
    pub downstream_headers: HashMap<String, String>,

    #[serde(default)]
    pub rewrite: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct CommandProxieConfig {
    pub target: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub downstream_headers: HashMap<String, String>,

    #[serde(default)]
    pub rewrite: HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum ProxieConfig {
    #[serde(rename = "remote")]
    Remote(RemoteProxieConfig),

    #[serde(rename = "local")]
    Local(LocalProxieConfig),

    #[serde(rename = "command")]
    Command(CommandProxieConfig),
}

pub fn load_config(explicit_path: Option<String>) -> AppConfig {
    let config_path = match explicit_path {
        Some(p) => PathBuf::from(p),
        None => {
            let mut path = dirs::config_dir().expect("Could not find config directory");
            path.push("deeria");
            path.push("config.toml");
            path
        }
    };

    if !config_path.exists() {
        create_default_config(&config_path);
    }

    let content = fs::read_to_string(&config_path)
        .expect(&format!("Failed to read config at {:?}", config_path));

    toml::from_str(&content).expect("Invalid config format")
}

fn create_default_config(path: &Path) {
    println!("Creating default configuration at: {:?}", path);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Could not create config directory");
    }

    let default_toml = r#"
[server]
host = "127.0.0.1"
port = 4242
"#;

    fs::write(path, default_toml.trim()).expect("Could not write default config file");
}

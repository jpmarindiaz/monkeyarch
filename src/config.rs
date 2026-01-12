use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub root_directory: PathBuf,
    pub static_directory: Option<PathBuf>,
    pub bind_address: String,
    pub port: u16,
    pub max_upload_size: u64,
    pub enable_delete: bool,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default = "default_root")]
    root_directory: String,

    #[serde(default)]
    static_directory: Option<String>,

    #[serde(default = "default_bind")]
    bind_address: String,

    #[serde(default = "default_port")]
    port: u16,

    #[serde(default = "default_max_upload")]
    max_upload_size: u64,

    #[serde(default = "default_enable_delete")]
    enable_delete: bool,
}

fn default_root() -> String {
    "/home/pi/media".into()
}

fn default_bind() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    8000
}

fn default_max_upload() -> u64 {
    100 * 1024 * 1024 // 100 MB
}

fn default_enable_delete() -> bool {
    true
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = config::Config::builder()
            .set_default("root_directory", "/home/pi/media")?
            .set_default("bind_address", "0.0.0.0")?
            .set_default("port", 8000_i64)?
            .set_default("max_upload_size", 104857600_i64)?
            .set_default("enable_delete", true)?
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("MONKEYARCH"))
            .build()?;

        let raw: RawConfig = cfg.try_deserialize()?;

        Ok(Config {
            root_directory: PathBuf::from(raw.root_directory),
            static_directory: raw.static_directory.map(PathBuf::from),
            bind_address: raw.bind_address,
            port: raw.port,
            max_upload_size: raw.max_upload_size,
            enable_delete: raw.enable_delete,
        })
    }
}

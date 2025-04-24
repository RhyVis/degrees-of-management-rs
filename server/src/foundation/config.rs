use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};
use tracing::{error, info};

const CONFIG_FILE_NAME: &str = "config.toml";

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config() -> Result<()> {
    CONFIG.get_or_init(|| {
        Config::load().unwrap_or_else(|err| {
            error!("Error loading config: {}", err);
            Config::create_default().unwrap()
        })
    });

    info!("Config loaded");

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub port: u16,
    pub data_dir: String,
    pub game_def: HashMap<String, GameDef>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 3000,
            data_dir: String::from("data"),
            game_def: HashMap::new(),
        }
    }
}

impl Config {
    fn get_config_path() -> Result<PathBuf> {
        let current_dir = env::current_dir()?;
        Ok(current_dir.join(CONFIG_FILE_NAME))
    }

    fn create_default() -> Result<Self> {
        let config = Self::default();
        let config_content = toml::to_string_pretty(&config)?;
        let config_path = Self::get_config_path()?;

        let mut file = File::create(config_path)?;
        file.write_all(config_content.as_bytes())?;

        dbg!("Created default config: {:?}", &config);

        Ok(config)
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Self::create_default();
        }

        let config_content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(err) => {
                error!("Cannot read config file: {}", err);
                return Self::create_default();
            }
        };

        let config: Config = match toml::from_str(&config_content) {
            Ok(config) => config,
            Err(err) => {
                error!("Cannot parse config file: {}", err);
                return Self::create_default();
            }
        };

        dbg!("Loaded config: {:?}", &config);

        if !&config.get_data_path().exists() {
            fs::create_dir_all(&config.get_data_path())?;
        }

        Ok(config)
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let config_content = toml::to_string_pretty(&self)?;
        let config_path = Self::get_config_path()?;

        let mut file = File::create(config_path)?;
        file.write_all(config_content.as_bytes())?;

        Ok(())
    }

    pub fn get_data_path(&self) -> PathBuf {
        PathBuf::from(&self.data_dir)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct GameDef {
    pub use_mods: bool,
}

impl Default for GameDef {
    fn default() -> Self {
        GameDef { use_mods: false }
    }
}

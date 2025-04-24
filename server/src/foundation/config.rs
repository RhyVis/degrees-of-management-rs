use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::{env, fs};
use tracing::{debug, error, info};

const CONFIG_FILE_NAME: &str = "config.toml";

pub static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init_config() -> Result<()> {
    CONFIG.get_or_init(|| {
        Config::load().unwrap_or_else(|err| {
            error!("Error loading config: {}", err);
            Config::create_default().unwrap_or_else(|err| {
                let path = Config::get_config_path().unwrap_or(PathBuf::new().join("!Unknown"));
                error!(
                    "Error creating default config on {}: {}",
                    path.to_string_lossy().to_string(),
                    err
                );
                panic!("Failed to create default config");
            })
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

        info!("Creating default config at {:?}", &config_path);

        let mut file = File::create(&config_path)?;
        file.write_all(config_content.as_bytes())?;

        debug!("Created default config at {:?}", config_path);

        Ok(config)
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            info!("Config does not exist, creating default config.");
            return Self::create_default();
        }

        match fs::read_to_string(&config_path) {
            Ok(config_content) => match toml::from_str::<Config>(&config_content) {
                Ok(config) => {
                    debug!("Loaded config: {:?}", &config);

                    if !&config.get_data_path().exists() {
                        fs::create_dir_all(&config.get_data_path())?;
                    }

                    Ok(config)
                }
                Err(err) => {
                    error!("Cannot parse config file: {}", err);
                    Self::create_default()
                }
            },
            Err(err) => {
                error!("Cannot read config file: {}", err);
                Self::create_default()
            }
        }
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct GameDef {
    pub name: Option<String>,
    pub use_mods: bool,
    pub use_save_sync_mod: bool,
}

impl Default for GameDef {
    fn default() -> Self {
        GameDef {
            name: None,
            use_mods: true,
            use_save_sync_mod: true,
        }
    }
}

impl GameDef {
    pub fn is_use_save_sync_mod(&self) -> bool {
        self.use_mods && self.use_save_sync_mod
    }
}

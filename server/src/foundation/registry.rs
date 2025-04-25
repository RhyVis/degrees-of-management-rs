use crate::foundation::config::{CONFIG, Config};
use crate::foundation::structure::{GameInfo, IndexInfo, InstanceInfo, LayerInfo, ModInfo};
use crate::util::file::{list_dir_name, list_filename_limit_extension};
use crate::util::vfs::{FileSystemTree, InstanceFS};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
use tracing::{debug, error, info, warn};

pub fn init_registry() -> Result<GameRegistry> {
    info!("Game registry initialized");
    let mut registry = GameRegistry::new();
    load_games(&mut registry)?;
    Ok(registry)
}

fn walk_game_dir(config: &Config) -> Result<()> {
    let data_dir = config.get_data_path();

    let defined_game_ids: Vec<String> = config.game_def.keys().cloned().collect();

    for defined_game_id in &defined_game_ids {
        let game_dir = data_dir.join(&defined_game_id);
        if !game_dir.exists() {
            fs::create_dir(&game_dir)?;
            info!(
                "Created directory for game '{}' at {:?}",
                &defined_game_id, &game_dir
            );
        }
    }

    let actual_dirs = fs::read_dir(&data_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .into_iter();

    for actual_dir in actual_dirs {
        if let Some(dir_name) = actual_dir.file_name().to_str() {
            if !defined_game_ids.contains(&dir_name.to_string()) {
                warn!(
                    "Directory '{}' exists but is not defined in the config",
                    dir_name
                );
            }
        }
    }

    Ok(())
}

fn load_games(registry: &mut GameRegistry) -> Result<()> {
    if let Some(config) = CONFIG.get() {
        if config.game_def.is_empty() {
            warn!(
                "No game definitions in 'game_def' found, consider adding some to the config file"
            );
            std::process::exit(666);
        } else {
            let data_dir = config.get_data_path();

            walk_game_dir(config)?;

            for (id, def) in &config.game_def {
                info!("Loading game: '{}'", id);

                let that_path = data_dir.join(id);
                let mut that_game = GameInfo::of(id, that_path, def.clone());

                debug!("Loading index for game: {}", id);
                load_index(&mut that_game)?;

                if def.use_mods {
                    debug!("Loading mod for game: {}", id);
                    load_mod(&mut that_game)?;
                }

                debug!("Loading layer for game: {}", id);
                load_layer(&mut that_game)?;
                debug!("Loading instance for game: {}", id);
                load_instance(&mut that_game)?;

                if !that_game.get_save_path().exists() {
                    fs::create_dir(&that_game.get_save_path())?;
                }

                registry.add(that_game);
            }
        }
    } else {
        return Err(anyhow!("Config not initialized"));
    }
    Ok(())
}

fn load_index(game: &mut GameInfo) -> Result<()> {
    let index_dir = game.get_index_path();
    if !index_dir.exists() {
        fs::create_dir(&index_dir)?;
    }
    let names = list_filename_limit_extension(&index_dir, Some("html"))?;
    let ids: Vec<String> = names.iter().map(|(id, _)| id.clone()).collect();

    if names.is_empty() {
        warn!("No index files found for '{}'", &game.id);
    } else {
        info!("Found index files: {:?} for '{}'", ids, &game.id);
    }

    for (id, file_name) in names {
        game.indexes
            .insert(id.clone(), IndexInfo::of(&id, &file_name, &index_dir));
    }

    Ok(())
}

fn load_layer(game: &mut GameInfo) -> Result<()> {
    let layer_dir = game.get_layer_path();
    if !layer_dir.exists() {
        fs::create_dir(&layer_dir)?;
    }
    let names = list_dir_name(&layer_dir)?;

    if names.is_empty() {
        warn!("No layer directory found for '{}'", &game.id);
    } else {
        info!("Found layer directories: {:?} for '{}'", names, &game.id);
    }

    for name in names {
        game.layers
            .insert(name.clone(), LayerInfo::of(&name, &layer_dir));
    }

    Ok(())
}

fn load_mod(game: &mut GameInfo) -> Result<()> {
    let mod_dir = game.get_mod_path();
    if !mod_dir.exists() {
        fs::create_dir(&mod_dir)?;
    }
    let names = list_filename_limit_extension(&mod_dir, Some("zip"))?;
    let ids: Vec<String> = names.iter().map(|(id, _)| id.clone()).collect();

    if names.is_empty() {
        warn!("No mod files found for '{}'", &game.id);
    } else {
        info!("Found mod files: {:?} for '{}'", ids, &game.id);
    }

    for (id, file_name) in names {
        game.mods
            .insert(id.clone(), ModInfo::of(&id, &file_name, &mod_dir));
    }

    Ok(())
}

fn load_instance(game: &mut GameInfo) -> Result<()> {
    let instance_dir = game.get_instance_path();
    if !instance_dir.exists() {
        fs::create_dir(&instance_dir)?;
    }

    let formats = vec!["json", "toml", "yaml"];
    let mut loaded_instances = 0;

    for format in formats {
        let files = list_filename_limit_extension(&instance_dir, Some(format))?;
        if !files.is_empty() {
            debug!(
                "Found {} {} instance files for '{}'",
                files.len(),
                format,
                &game.id
            );

            for (_, file_name) in files {
                let path_to_file = instance_dir.join(&file_name);
                match load_instance_from_file(&path_to_file, format) {
                    Ok(instance) => {
                        let id = instance.id.clone();
                        if process_loaded_instance(game, instance)? {
                            loaded_instances += 1;
                        } else {
                            warn!(
                                "Instance '{}' defined in '{}' already loaded, skipping.",
                                id, file_name
                            );
                        }
                    }
                    Err(e) => {
                        error!("Failed to load instance file '{}': {}", file_name, e);
                    }
                }
            }
        }
    }

    if loaded_instances == 0 {
        warn!("No instance files found for '{}'", &game.id);
    } else {
        info!("Loaded {} instances for '{}'", loaded_instances, &game.id);
    }

    Ok(())
}

fn load_instance_from_file(path: &std::path::Path, format: &str) -> Result<InstanceInfo> {
    let content = fs::read_to_string(path)?;

    match format {
        "json" => serde_json::from_str(&content).map_err(|e| anyhow!("JSON parse error: {}", e)),
        "toml" => toml::from_str(&content).map_err(|e| anyhow!("TOML parse error: {}", e)),
        "yaml" => serde_yaml::from_str(&content).map_err(|e| anyhow!("YAML parse error: {}", e)),
        _ => Err(anyhow!("Unsupported format: {}", format)),
    }
}

fn process_loaded_instance(game: &mut GameInfo, mut instance: InstanceInfo) -> Result<bool> {
    if game.instances.contains_key(&instance.id) {
        return Ok(false);
    }

    let mut layer_fs_collection = Vec::with_capacity(instance.layers.len());

    for layer_id in &instance.layers {
        match game.layers.get_mut(layer_id) {
            Some(layer_info) => match layer_info.get_fs() {
                Ok(layer_fs) => layer_fs_collection.push(layer_fs.clone()),
                Err(err) => {
                    error!(
                        "Failed to create layer file system for {}: {}",
                        layer_id, err
                    );
                    return Err(anyhow!(
                        "Failed to create layer file system for {}: {}",
                        layer_id,
                        err
                    ));
                }
            },
            None => {
                let error_msg = format!(
                    "Layer {} referenced by instance {} not found",
                    &instance.id, layer_id
                );
                error!("{}", error_msg);
                return Err(anyhow!(error_msg));
            }
        }
    }

    let instance_fs = InstanceFS::new(&instance.id, layer_fs_collection);
    let stats = instance_fs.get_node_stats();

    info!(
        "Created instance '{}' fs contains {} nodes ({} dirs, {} files)",
        &instance.id, stats.total, stats.dirs, stats.files
    );

    instance.fs = Some(instance_fs);
    game.instances.insert(instance.id.clone(), instance);

    Ok(true)
}

pub struct GameRegistry {
    registry: HashMap<String, GameInfo>,
}

impl GameRegistry {
    pub fn new() -> Self {
        GameRegistry {
            registry: HashMap::new(),
        }
    }
}

pub trait Registry<T> {
    fn add(&mut self, item: T);
    fn get(&self, id: &str) -> Option<&T>;

    fn all(&self) -> Vec<(String, &T)>;
}

impl Registry<GameInfo> for GameRegistry {
    fn add(&mut self, item: GameInfo) {
        self.registry.insert(item.id.clone(), item);
    }

    fn get(&self, id: &str) -> Option<&GameInfo> {
        self.registry.get(id)
    }

    fn all(&self) -> Vec<(String, &GameInfo)> {
        self.registry
            .iter()
            .map(|(id, game)| (id.clone(), game))
            .collect()
    }
}

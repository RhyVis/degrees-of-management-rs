use crate::foundation::config::GameDef;
use crate::util::resolve::{InstanceFS, LayerFS};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct GameInfo {
    pub id: String,
    pub path: PathBuf,
    pub instances: HashMap<String, InstanceInfo>,
    pub indexes: HashMap<String, IndexInfo>,
    pub layers: HashMap<String, LayerInfo>,
    pub mods: HashMap<String, ModInfo>,

    pub instance_fs: HashMap<String, InstanceFS>,
    pub game_def: GameDef,
}

impl GameInfo {
    pub fn of(id: &str, path: PathBuf, def_copy: GameDef) -> Self {
        GameInfo {
            id: id.to_string(),
            path,
            instances: HashMap::new(),
            indexes: HashMap::new(),
            layers: HashMap::new(),
            mods: HashMap::new(),
            instance_fs: HashMap::new(),
            game_def: def_copy,
        }
    }

    pub fn get_index_path(&self) -> PathBuf {
        self.path.join("index")
    }

    pub fn get_layer_path(&self) -> PathBuf {
        self.path.join("layer")
    }

    pub fn get_mod_path(&self) -> PathBuf {
        self.path.join("mod")
    }

    pub fn get_instance_path(&self) -> PathBuf {
        self.path.join("instance")
    }

    pub fn get_save_path(&self) -> PathBuf {
        self.path.join("save")
    }

    pub fn get_save_path_append(&self, next_dir: &str) -> PathBuf {
        self.path.join("save").join(next_dir)
    }
}

pub trait FileInfo {
    fn get_id(&self) -> &str;
    fn get_path(&self) -> &Path;

    fn is_file(&self) -> bool {
        self.get_path().is_file()
    }

    fn read_bytes(&self) -> Result<Vec<u8>> {
        if !self.is_file() {
            return Err(anyhow::anyhow!("FileInfo '{}' not a file", self.get_id()));
        }
        fs::read(self.get_path()).map_err(Error::from)
    }
}

pub struct IndexInfo {
    pub id: String,
    pub path: PathBuf,
}

impl IndexInfo {
    pub fn of(id: &str, file_name: &str, base_path: &Path) -> Self {
        Self {
            id: id.to_string(),
            path: base_path.join(file_name),
        }
    }
}

impl FileInfo for IndexInfo {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_path(&self) -> &Path {
        &self.path
    }
}

pub struct LayerInfo {
    pub id: String,
    pub path: PathBuf,
    fs: Option<LayerFS>,
}

impl FileInfo for LayerInfo {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_path(&self) -> &Path {
        &self.path
    }
}

impl LayerInfo {
    pub fn of(id: &str, base_path: &Path) -> Self {
        Self {
            id: id.to_string(),
            path: base_path.join(id),
            fs: None,
        }
    }

    pub fn get_fs(&mut self) -> Result<&LayerFS> {
        if self.fs.is_none() {
            self.fs = Some(LayerFS::new(&self.id, &self.path)?);
        }
        Ok(self.fs.as_ref().unwrap())
    }
}

pub struct ModInfo {
    pub id: String,
    pub path: PathBuf,
}

impl FileInfo for ModInfo {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_path(&self) -> &Path {
        &self.path
    }
}

impl ModInfo {
    pub fn of(id: &str, file_name: &str, base_path: &Path) -> Self {
        Self {
            id: id.to_string(),
            path: base_path.join(file_name),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstanceInfo {
    pub id: String,
    pub index: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub mods: Vec<String>,
    #[serde(default)]
    pub layers: Vec<String>,
}

impl InstanceInfo {
    pub fn get_name(&self) -> String {
        if let Some(name) = &self.name {
            name.clone()
        } else {
            self.id.clone()
        }
    }
}

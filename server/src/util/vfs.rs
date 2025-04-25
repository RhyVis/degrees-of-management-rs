use anyhow::Result;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tracing::info;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug, Clone)]
pub enum FSNode {
    File {
        id: u64,
        real_path: Arc<PathBuf>,
    },
    Directory {
        id: u64,
        children: HashMap<String, Arc<FSNode>>,
    },
}

impl FSNode {
    pub fn new_file(real_path: PathBuf) -> Arc<Self> {
        let path_str = real_path.to_string_lossy();
        let id = xxh3_64(path_str.as_bytes());

        Arc::new(FSNode::File {
            id,
            real_path: Arc::new(real_path),
        })
    }

    pub fn new_directory(children: HashMap<String, Arc<Self>>) -> Arc<Self> {
        let mut hasher = xxh3_64(b"dir");
        for (name, child) in &children {
            hasher ^= xxh3_64(name.as_bytes());
            hasher = hasher.rotate_left(7) ^ child.get_id();
        }

        Arc::new(FSNode::Directory {
            id: hasher,
            children,
        })
    }

    pub fn get_id(&self) -> u64 {
        match self {
            FSNode::File { id, .. } => *id,
            FSNode::Directory { id, .. } => *id,
        }
    }

    #[allow(dead_code)]
    pub fn equals(&self, other: &FSNode) -> bool {
        self.get_id() == other.get_id()
    }
}

#[derive(Debug)]
pub struct NodeStats {
    pub files: usize,
    pub dirs: usize,
    pub total: usize,
}

pub trait FileSystemTree {
    fn get_root(&self) -> &FSNode;

    fn get_node_stats(&self) -> NodeStats {
        Self::calculate_node_stats(self.get_root())
    }

    fn calculate_node_stats(node: &FSNode) -> NodeStats {
        match node {
            FSNode::File { .. } => NodeStats {
                files: 1,
                dirs: 0,
                total: 1,
            },
            FSNode::Directory { children, .. } => {
                let mut stats = NodeStats {
                    files: 0,
                    dirs: 1,
                    total: 1,
                };

                for child in children.values() {
                    let child_stats = Self::calculate_node_stats(child);
                    stats.files += child_stats.files;
                    stats.dirs += child_stats.dirs;
                    stats.total += child_stats.total;
                }

                stats
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayerFS {
    pub root: Arc<FSNode>,
}

impl LayerFS {
    pub fn new(id: &str, layer_path: &Path) -> Result<Self> {
        let start = Instant::now();

        let root = FSNode::new_directory(HashMap::new());
        let mut layer_fs = LayerFS { root };

        layer_fs.build_tree(layer_path)?;

        let elapsed = start.elapsed();
        info!("LayerFS '{}' created in {:.2?}", id, elapsed);

        Ok(layer_fs)
    }

    fn build_tree(&mut self, path: &Path) -> Result<()> {
        if let FSNode::Directory { children, .. } = Arc::make_mut(&mut self.root) {
            Self::build_tree_recursive(path, path, children)?;
        }
        Ok(())
    }

    fn build_tree_recursive(
        base_path: &Path,
        current_path: &Path,
        children: &mut HashMap<String, Arc<FSNode>>,
    ) -> Result<()> {
        let entries: Vec<_> = fs::read_dir(current_path)?
            .filter_map(|entry| entry.ok())
            .collect();
        let mut results = HashMap::with_capacity(entries.len());

        let new_entries: Vec<_> = entries
            .par_iter()
            .map(|entry| {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                if path.is_dir() {
                    let mut dir_children = HashMap::new();
                    let _ = Self::build_tree_recursive(base_path, &path, &mut dir_children);

                    let node = FSNode::new_directory(dir_children);
                    (file_name, node)
                } else {
                    let node = FSNode::new_file(path);
                    (file_name, node)
                }
            })
            .collect();

        results.extend(new_entries);
        children.extend(results);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct InstanceFS {
    pub root: Arc<FSNode>,
}

impl InstanceFS {
    pub fn new(id: &str, layers: Vec<LayerFS>) -> Self {
        let start = Instant::now();

        let root = if let Some(last_layer) = layers.last() {
            last_layer.root.clone()
        } else {
            FSNode::new_directory(HashMap::new())
        };

        let mut instance_fs = InstanceFS { root };

        for layer in layers.iter().rev().skip(1) {
            instance_fs.merge_layer(layer);
        }

        let elapsed = start.elapsed();
        info!(
            "InstanceFS '{}' created in {:.2?} with {} layers",
            id,
            elapsed,
            layers.len()
        );

        instance_fs
    }

    fn merge_layer(&mut self, layer: &LayerFS) {
        if let (
            FSNode::Directory {
                children: target_children,
                ..
            },
            FSNode::Directory {
                children: source_children,
                ..
            },
        ) = (&mut *Arc::make_mut(&mut self.root), &*layer.root)
        {
            Self::merge_directories(target_children, source_children);
        }
    }

    fn merge_directories(
        target: &mut HashMap<String, Arc<FSNode>>,
        source: &HashMap<String, Arc<FSNode>>,
    ) {
        for (name, source_node) in source {
            match &**source_node {
                FSNode::Directory { .. } => {
                    // Same name dir
                    if let Some(target_node) = target.get_mut(name) {
                        if let FSNode::Directory {
                            children: target_children,
                            ..
                        } = &mut *Arc::make_mut(target_node)
                        {
                            if let FSNode::Directory {
                                children: ref source_dir_children,
                                ..
                            } = **source_node
                            {
                                Self::merge_directories(target_children, source_dir_children);
                            }
                        } else {
                            // Target is a file, replace it with the directory
                            target.insert(name.clone(), Arc::clone(source_node));
                        }
                    } else {
                        // Skip cloning, insert directly
                        target.insert(name.clone(), Arc::clone(source_node));
                    }
                }
                FSNode::File { .. } => {
                    // File always replaces the target
                    target.insert(name.clone(), Arc::clone(source_node));
                }
            }
        }
    }

    pub fn resolve_path(&self, virtual_path: &str) -> Option<PathBuf> {
        let mut current_node = &*self.root;

        for part in virtual_path.split('/').filter(|p| !p.is_empty()) {
            match current_node {
                FSNode::Directory { children, .. } => {
                    if let Some(node) = children.get(part) {
                        current_node = &**node;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }

        match current_node {
            FSNode::File { real_path, .. } => Some((**real_path).clone()),
            _ => None,
        }
    }
}

impl FileSystemTree for LayerFS {
    fn get_root(&self) -> &FSNode {
        &*self.root
    }
}

impl FileSystemTree for InstanceFS {
    fn get_root(&self) -> &FSNode {
        &*self.root
    }
}

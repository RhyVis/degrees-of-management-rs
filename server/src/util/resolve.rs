use anyhow::Result;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

#[derive(Debug, Clone)]
pub enum FSNode {
    File { real_path: PathBuf },
    Directory { children: HashMap<String, FSNode> },
}

#[derive(Debug, Clone)]
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
    pub root: FSNode,
}

impl LayerFS {
    pub fn new(id: &str, layer_path: &Path) -> Result<Self> {
        let start = Instant::now();

        let root = FSNode::Directory {
            children: HashMap::new(),
        };

        let mut layer_fs = LayerFS { root };

        layer_fs.build_tree(layer_path)?;

        let elapsed = start.elapsed();
        info!("LayerFS '{}' created in {:.2?}", id, elapsed);

        Ok(layer_fs)
    }

    fn build_tree(&mut self, path: &Path) -> Result<()> {
        if let FSNode::Directory {
            ref mut children, ..
        } = self.root
        {
            Self::build_tree_recursive(path, path, children)?;
        }
        Ok(())
    }

    fn build_tree_recursive(
        base_path: &Path,
        current_path: &Path,
        children: &mut HashMap<String, FSNode>,
    ) -> Result<()> {
        let entries: Vec<_> = fs::read_dir(current_path)?
            .filter_map(|entry| entry.ok())
            .collect();

        let results: HashMap<_, _> = entries
            .par_iter()
            .map(|entry| {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                if path.is_dir() {
                    let mut dir_children = HashMap::new();
                    let _ = Self::build_tree_recursive(base_path, &path, &mut dir_children);

                    (
                        file_name.clone(),
                        FSNode::Directory {
                            children: dir_children,
                        },
                    )
                } else {
                    (file_name.clone(), FSNode::File { real_path: path })
                }
            })
            .collect();

        children.extend(results);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct InstanceFS {
    pub root: FSNode,
}

impl InstanceFS {
    pub fn new(id: &str, layers: Vec<LayerFS>) -> Self {
        let start = Instant::now();

        let root = FSNode::Directory {
            children: HashMap::new(),
        };

        let mut instance_fs = InstanceFS { root };

        for layer in &layers {
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
            FSNode::Directory { children, .. },
            FSNode::Directory {
                children: layer_children,
                ..
            },
        ) = (&mut self.root, &layer.root)
        {
            Self::merge_directories(children, layer_children);
        }
    }

    fn merge_directories(target: &mut HashMap<String, FSNode>, source: &HashMap<String, FSNode>) {
        for (name, node) in source {
            match node {
                FSNode::Directory {
                    children: source_children,
                    ..
                } => {
                    // 如果目标中已存在同名目录，则递归合并
                    if let Some(FSNode::Directory {
                        children: target_children,
                        ..
                    }) = target.get_mut(name)
                    {
                        Self::merge_directories(target_children, source_children);
                    } else {
                        // 否则直接克隆整个目录结构
                        target.insert(name.clone(), node.clone());
                    }
                }
                FSNode::File { .. } => {
                    // 文件总是覆盖
                    target.insert(name.clone(), node.clone());
                }
            }
        }
    }

    // 查找文件的实际路径
    pub fn resolve_path(&self, virtual_path: &str) -> Option<PathBuf> {
        let parts: Vec<&str> = virtual_path.split('/').filter(|p| !p.is_empty()).collect();
        self.resolve_path_parts(&parts, &self.root)
    }

    fn resolve_path_parts(&self, parts: &[&str], node: &FSNode) -> Option<PathBuf> {
        if parts.is_empty() {
            return None;
        }

        match node {
            FSNode::Directory { children, .. } => {
                if let Some(child) = children.get(parts[0]) {
                    if parts.len() == 1 {
                        if let FSNode::File { real_path, .. } = child {
                            return Some(real_path.clone());
                        }
                    } else {
                        return self.resolve_path_parts(&parts[1..], child);
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl FileSystemTree for LayerFS {
    fn get_root(&self) -> &FSNode {
        &self.root
    }
}

impl FileSystemTree for InstanceFS {
    fn get_root(&self) -> &FSNode {
        &self.root
    }
}

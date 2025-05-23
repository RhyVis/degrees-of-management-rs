use anyhow::Result;
use axum::http::header::{CACHE_CONTROL, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use std::fs;
use std::path::Path;

pub fn list_filename_limit_extension(
    path: &Path,
    extension: Option<&str>,
) -> Result<Vec<(String, String)>> {
    if !path.exists() || !path.is_dir() {
        return Ok(Vec::new());
    }

    let extension = extension.map(|s| s.to_ascii_lowercase());

    let entries = fs::read_dir(path);
    let mut names = Vec::new();

    for entry in entries? {
        let entry = entry?;
        let entry_path = entry.path();

        if !entry_path.is_file() {
            continue;
        }
        if let Some(req_ext) = &extension {
            if let Some(ext) = entry_path.extension() {
                if ext.to_ascii_lowercase().to_string_lossy().to_string() == req_ext.to_string() {
                    let file_name = entry_path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    let file_stem = entry_path
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    names.push((file_stem, file_name));
                }
            }
        } else {
            let file_name = entry_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let file_stem = entry_path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            names.push((file_stem, file_name));
        }
    }

    Ok(names)
}

pub fn list_dir_name(path: &Path) -> Result<Vec<String>> {
    if !path.exists() || !path.is_dir() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(path)?;
    let mut names = Vec::new();

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if let Some(dir_name) = entry_path.file_name() {
                let dir_name = dir_name.to_string_lossy().to_string();
                names.push(dir_name);
            }
        }
    }

    Ok(names)
}

pub fn etag_hash(content: &Vec<u8>) -> String {
    format!("\"{}\"", xxhash_rust::xxh3::xxh3_64(content))
}

pub fn etag_check(content: &Vec<u8>, headers: &HeaderMap) -> Option<Response> {
    let etag_val = etag_hash(content);

    if let Some(if_not_match) = headers.get(IF_NONE_MATCH) {
        if let Ok(cli_tag) = if_not_match.to_str() {
            if cli_tag == etag_val {
                return Some(
                    (
                        StatusCode::NOT_MODIFIED,
                        [
                            (CACHE_CONTROL, "public, max-age=31536000"),
                            (ETAG, etag_val.as_str()),
                        ],
                    )
                        .into_response(),
                );
            }
        }
    }

    None
}

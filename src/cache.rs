use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use serde::{de::DeserializeOwned, Serialize};
use log::info;

// Retourne le chemin du répertoire du cache, le crée s'il n'existe pas.
fn get_cache_dir() -> Result<PathBuf, std::io::Error> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cache directory not found"))?
        .join("nlauncher");

    fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

// Retourne le chemin complet du fichier cache.
fn get_cache_file_path() -> Result<PathBuf, std::io::Error> {
    get_cache_dir().map(|dir| dir.join("app_ids.json"))
}

// Charge les données depuis un fichier JSON.
pub fn load_from_cache<T: DeserializeOwned>() -> Option<T> {
    let path = get_cache_file_path().ok()?;
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    serde_json::from_str(&contents).ok()
}

// Sauvegarde les données dans un fichier JSON.
pub fn save_to_cache<T: Serialize>(data: &T) -> Result<(), std::io::Error> {
    let path = get_cache_file_path()?;
    let json_data = serde_json::to_string_pretty(data)?;
    let mut file = File::create(path)?;
    file.write_all(json_data.as_bytes())
}

pub fn clear_cache() -> Result<(), std::io::Error> {
    if let Ok(path) = get_cache_file_path() {
        if path.exists() {
            info!("Clearing cache file at: {path:?}");
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

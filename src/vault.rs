use anyhow::{Context, Result};
use keepass::{db::Node, Database, DatabaseKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_DURATION_SECS: u64 = 600;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Settings {
    vault_path: Option<String>,
}

impl Settings {
    fn load() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("nlauncher");
        
        let config_file = config_dir.join("settings.json");
        
        fs::read_to_string(config_file)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct VaultEntry {
    pub title: String,
    pub username: String,
    pub password: String,
    pub totp: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Session {
    unlocked_at: u64,
    password: String,
}

pub struct VaultManager {
    session_file: PathBuf,
}

impl VaultManager {
    pub fn new() -> Self {
        let session_file = std::env::temp_dir().join("nlauncher_vault_session");
        Self { session_file }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn is_unlocked(&self) -> bool {
        self.load_session().is_some()
    }

    fn load_session(&self) -> Option<Session> {
        let data = fs::read_to_string(&self.session_file).ok()?;
        let session: Session = serde_json::from_str(&data).ok()?;
        
        if Self::current_timestamp() - session.unlocked_at < SESSION_DURATION_SECS {
            Some(session)
        } else {
            let _ = fs::remove_file(&self.session_file);
            None
        }
    }

    fn save_session(&self, password: &str) -> Result<()> {
        let session = Session {
            unlocked_at: Self::current_timestamp(),
            password: password.to_string(),
        };
        fs::write(&self.session_file, serde_json::to_string(&session)?)?;
        Ok(())
    }

    pub fn unlock(&self, password: &str) -> Result<Vec<VaultEntry>> {
        let settings = Settings::load();
        let vault_path = settings.vault_path
            .context("No vault configured. Open settings to set vault path.")?;
        
        let db = Database::open(
            &mut fs::File::open(&vault_path)?,
            DatabaseKey::new().with_password(password),
        )?;

        self.save_session(password)?;
        Ok(self.extract_entries(&db))
    }

    pub fn search(&self, query: &str) -> Result<Vec<VaultEntry>> {
        let session = self.load_session().context("Vault locked")?;
        let settings = Settings::load();
        let vault_path = settings.vault_path
            .context("No vault configured. Open settings to set vault path.")?;
        
        let db = Database::open(
            &mut fs::File::open(&vault_path)?,
            DatabaseKey::new().with_password(&session.password),
        )?;

        let entries = self.extract_entries(&db);
        Ok(entries
            .into_iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&query.to_lowercase())
                    || e.username.to_lowercase().contains(&query.to_lowercase())
            })
            .collect())
    }

    fn extract_entries(&self, db: &Database) -> Vec<VaultEntry> {
        let mut entries = Vec::new();
        for node in &db.root.children {
            self.traverse_group(node, &mut entries);
        }
        entries
    }

    fn traverse_group(&self, node: &Node, entries: &mut Vec<VaultEntry>) {
        match node {
            Node::Group(g) => {
                for child in &g.children {
                    self.traverse_group(child, entries);
                }
            }
            Node::Entry(e) => {
                let title = e.get_title().unwrap_or("Untitled").to_string();
                let username = e.get_username().unwrap_or("").to_string();
                let password = e.get_password().unwrap_or("").to_string();
                let totp = e.get("otp").map(|s| s.to_string());

                entries.push(VaultEntry {
                    title,
                    username,
                    password,
                    totp,
                });
            }
        }
    }
}

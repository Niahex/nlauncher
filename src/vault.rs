use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_DURATION_SECS: u64 = 600;

#[derive(Serialize, Deserialize, Clone, Default)]
struct Settings {
    vault_path: Option<String>,
}

impl Settings {}

#[derive(Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub title: String,
    pub username: String,
    pub password: String,
    pub totp: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Session {
    unlocked_at: u64,
    keepassxc_pid: Option<u32>,
}

pub struct VaultManager {
    session_file: PathBuf,
}

impl Clone for VaultManager {
    fn clone(&self) -> Self {
        Self {
            session_file: self.session_file.clone(),
        }
    }
}

impl VaultManager {
    pub fn new() -> Self {
        let tmp_dir = std::env::temp_dir();
        Self {
            session_file: tmp_dir.join("nlauncher_keepassxc_session"),
        }
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

        // Check if session is still valid and KeePassXC is running
        if Self::current_timestamp() - session.unlocked_at < SESSION_DURATION_SECS {
            if let Some(pid) = session.keepassxc_pid {
                if std::path::Path::new(&format!("/proc/{pid}")).exists() {
                    return Some(session);
                }
            }
        }

        let _ = fs::remove_file(&self.session_file);
        None
    }

    fn save_session(&self, pid: u32) -> Result<()> {
        let session = Session {
            unlocked_at: Self::current_timestamp(),
            keepassxc_pid: Some(pid),
        };
        fs::write(&self.session_file, serde_json::to_string(&session)?)?;
        Ok(())
    }

    pub fn unlock(&self, _password: &str) -> Result<Vec<VaultEntry>> {
        // Check if KeePassXC is running and unlocked via DBus
        // If not, return error asking user to open KeePassXC

        match self.load_from_session() {
            Ok(entries) => {
                self.save_session(0)?;
                Ok(entries)
            }
            Err(e) => {
                Err(anyhow::anyhow!(
                    "Please open and unlock KeePassXC first. Enable 'Secret Service Integration' in KeePassXC settings. Error: {e}"
                ))
            }
        }
    }

    pub fn load_from_session(&self) -> Result<Vec<VaultEntry>> {
        use secret_service::blocking::SecretService;
        use secret_service::EncryptionType;

        eprintln!("[vault] Connecting to Secret Service...");
        let ss = SecretService::connect(EncryptionType::Plain)?;

        eprintln!("[vault] Getting default collection...");
        let collection = ss.get_default_collection()?;

        eprintln!("[vault] Checking if locked...");
        if collection.is_locked()? {
            return Err(anyhow::anyhow!("KeePassXC is locked"));
        }

        eprintln!("[vault] Getting all items...");
        let items = collection.get_all_items()?;
        eprintln!("[vault] Found {} items", items.len());

        let mut entries = Vec::new();

        for (i, item) in items.iter().enumerate() {
            eprintln!("[vault] Processing item {}/{}", i + 1, items.len());
            let label = item.get_label()?;
            let secret = item.get_secret()?;
            let attributes = item.get_attributes()?;

            let username = attributes
                .get("username")
                .or_else(|| attributes.get("UserName"))
                .cloned()
                .unwrap_or_default();

            entries.push(VaultEntry {
                title: label,
                username,
                password: String::from_utf8_lossy(&secret).to_string(),
                totp: None,
            });
        }

        eprintln!("[vault] Done! Loaded {} entries", entries.len());
        Ok(entries)
    }
}

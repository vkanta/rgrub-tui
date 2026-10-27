use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrubSettings {
    pub entries: HashMap<String, String>,
}

impl GrubSettings {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.entries.insert(key, value);
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut content = String::new();
        // Sort keys to keep the file predictable
        let mut keys: Vec<_> = self.entries.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(value) = self.entries.get(key) {
                content.push_str(&format!("{}={}\n", key, value));
            }
        }

        fs::write(path, content).context("Failed to write settings file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_save() {
        let mut settings = GrubSettings::new();
        settings.set("GRUB_TIMEOUT".to_string(), "5".to_string());
        settings.set("GRUB_DEFAULT".to_string(), "0".to_string());

        let test_file = "test_grub_settings.cfg";
        settings.save(test_file).unwrap();

        let content = fs::read_to_string(test_file).unwrap();
        assert!(content.contains("GRUB_DEFAULT=0"));
        assert!(content.contains("GRUB_TIMEOUT=5"));

        fs::remove_file(test_file).unwrap();
    }
}

use serde::{Deserialize, Serialize};
use std::fs;

use crate::error::{SammyError, SammyResult};
use crate::paths::{brains_config_path, ensure_dirs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainEntry {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub base_url: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub prefer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainConfigFile {
    pub brains: Vec<BrainEntry>,
}

impl BrainConfigFile {
    pub fn default_config() -> Self {
        Self {
            brains: vec![
                BrainEntry {
                    id: "kobold".into(),
                    kind: "kobold".into(),
                    label: "Local (Kobold :5001)".into(),
                    base_url: "http://127.0.0.1:5001".into(),
                    model: None,
                    prefer: true,
                },
                BrainEntry {
                    id: "venice".into(),
                    kind: "venice".into(),
                    label: "Venice (cloud)".into(),
                    base_url: "https://api.venice.ai/api/v1".into(),
                    model: Some("llama-3.3-70b".into()),
                    prefer: false,
                },
            ],
        }
    }

    pub fn load_or_default() -> Self {
        let _ = ensure_dirs();
        let path = brains_config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(raw) => match toml::from_str::<BrainConfigFile>(&raw) {
                    Ok(cfg) if !cfg.brains.is_empty() => return cfg,
                    _ => {}
                },
                Err(_) => {}
            }
        }
        let cfg = Self::default_config();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> SammyResult<()> {
        ensure_dirs()?;
        let raw = toml::to_string_pretty(self)
            .map_err(|e| SammyError::msg(format!("toml: {e}")))?;
        fs::write(brains_config_path(), raw)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&BrainEntry> {
        self.brains.iter().find(|b| b.id == id)
    }
}

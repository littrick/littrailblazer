use crate::deploy::{InstallItem, Installed};
use anyhow::Ok;
use log::info;
use regex::Regex;

#[derive(Debug)]
pub struct Env {
    key: String,
    value: String,
}

impl Env {
    pub fn from_kv<K, V>(key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl InstallItem for Env {
    fn check(&self) -> anyhow::Result<()> {
        info!(target: "Env", "Checking env key {}...", self.key);

        let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
        anyhow::ensure!(
            !self.key.is_empty() && !self.key.contains('=') && re.is_match(&self.key),
            "Name only allows letters, numbers, underscores(_), and hyphens(-)."
        );
        Ok(())
    }

    fn install(&self) -> anyhow::Result<Installed> {
        info!(target: "Env", "Installing {}...", self.key);
        Ok(Installed::Rc {
            command: format!("export {}=\"{}\"", self.key, self.value),
        })
    }
}

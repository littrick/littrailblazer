use log::info;
use regex::Regex;

use crate::deploy::{InstallItem, Installed};

#[derive(Debug)]
pub struct Alias {
    name: String,
    command: String,
}

impl Alias {
    pub fn from_pair<N, C>(name: N, command: C) -> Self
    where
        N: Into<String>,
        C: Into<String>,
    {
        Self {
            name: name.into(),
            command: command.into(),
        }
    }
}

impl InstallItem for Alias {
    fn check(&self) -> anyhow::Result<()> {
        info!(target: "Alias", "Checking alias {}", self.name);

        let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
        anyhow::ensure!(
            !self.name.is_empty() && !self.name.contains('=') && re.is_match(&self.name),
            "Name only allows letters, numbers, underscores(_), and hyphens(-)."
        );
        Ok(())
    }

    fn install(&self) -> anyhow::Result<Installed> {
        info!(target: "Alias", "Installing {}...", self.name);
        Ok(Installed::Rc {
            command: format!(
                "alias {}=$'{}'",
                self.name,
                self.command.replace("'", "\\'")
            ),
        })
    }
}

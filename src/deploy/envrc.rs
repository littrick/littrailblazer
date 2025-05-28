use crate::{
    deploy::{InstallItem, Installed},
    op::file::FileOp,
    types::config::Content,
};
use anyhow::{Context, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Envrc {
    content: Content,
    config_path: PathBuf,
}

impl Envrc {
    pub fn from_content<P>(content: &Content, config_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            content: content.clone(),
            config_path: config_path.as_ref().to_path_buf(),
        }
    }
}

impl InstallItem for Envrc {
    fn check(&self) -> anyhow::Result<()> {
        match &self.content {
            Content::Raw(_) => {}
            Content::File(path) => {
                let path = self.config_path.join(path);
                ensure!(
                    FileOp::exist(&path),
                    "{} is not exist",
                    path.to_string_lossy()
                );
                ensure!(
                    FileOp::exist(&path),
                    "{} is not a file",
                    path.to_string_lossy()
                );
            }
            Content::Url(_) => todo!(),
        }
        Ok(())
    }

    fn install(&self) -> anyhow::Result<Installed> {
        let rc_content = match &self.content {
            Content::Raw(content) => content.clone(),
            Content::File(path) => fs::read_to_string(path)
                .context(format!("Fail to read file {}", path.to_string_lossy()))?,
            Content::Url(_) => todo!(),
        };

        Ok(Installed::Rc {
            command: rc_content,
        })
    }
}

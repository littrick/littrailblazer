use std::path::{Path, PathBuf};

use crate::{
    deploy::{InstallItem, Installed},
    op::file::FileOp,
    types::config::{Content, StringOr},
};
use anyhow::{Context, ensure};

#[derive(Debug)]
pub struct File {
    content: StringOr<Content>,
    config_path: PathBuf,
    install_file: PathBuf,
}

impl File {
    pub fn from_content<P1, P2, P3>(
        path: P1,
        content: &StringOr<Content>,
        config_path: P2,
        install_path: P3,
    ) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        P3: AsRef<Path>,
    {
        let install_file = {
            if path.as_ref().is_relative() {
                install_path.as_ref().join(path)
            } else {
                path.as_ref().to_path_buf()
            }
        };

        Self {
            content: content.clone(),
            config_path: config_path.as_ref().to_path_buf(),
            install_file,
        }
    }
}

impl InstallItem for File {
    fn check(&self) -> anyhow::Result<()> {
        match &self.content {
            StringOr::String(_) | StringOr::Object(Content::Raw(_)) => {}
            StringOr::Object(Content::File(path)) => {
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
            StringOr::Object(Content::Url(_)) => todo!(),
        }

        ensure!(
            FileOp::is_file(&self.install_file) || !FileOp::is_dir(&self.install_file),
            "{} is a directory",
            self.install_file.to_string_lossy()
        );

        FileOp::write(&self.install_file, "", None).context(format!(
            "Fail to create file{}",
            self.install_file.to_string_lossy()
        ))?;

        Ok(())
    }

    fn install(&self) -> anyhow::Result<Installed> {
        if let Some(dir) = self.install_file.parent() {
            FileOp::mkdir(dir).context(format!("Fail to mkdir {}", dir.to_string_lossy()))?;
        }

        match &self.content {
            StringOr::String(content) | StringOr::Object(Content::Raw(content)) => {
                FileOp::write(&self.install_file, content, Some(0o755)).context(format!(
                    "fail to install file {}",
                    self.install_file.to_string_lossy()
                ))?;
            }
            StringOr::Object(Content::File(path)) => {
                FileOp::copy(path, &self.install_file).context(format!(
                    "fail to install file {}",
                    self.install_file.to_string_lossy()
                ))?;
            }
            StringOr::Object(Content::Url(_)) => todo!(),
        }

        Ok(Installed::File {
            file: self.install_file.to_path_buf(),
        })
    }
}

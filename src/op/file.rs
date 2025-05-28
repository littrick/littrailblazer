use anyhow::{Context, Ok, Result, ensure};
use std::{
    fs::{self},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FileOp;

#[allow(unused)]
impl FileOp {
    pub fn exist<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }

    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }

    pub fn mkdir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)
                .context(format!("Fail to create dir {}", path.to_string_lossy()))?;
        }
        ensure!(
            path.is_dir(),
            "{} is not a directory",
            path.to_string_lossy()
        );
        Ok(())
    }

    pub fn is_rel_file<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        path.is_file() && path.is_relative()
    }

    pub fn copy<P1, P2>(file: P1, target: P2) -> Result<PathBuf>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let file = file.as_ref();
        let target = target.as_ref();

        if let Some(dir) = target.parent()
            && !dir.exists()
        {
            fs::create_dir_all(dir)
                .context(format!("Fail to create dir {}", dir.to_string_lossy()))?;
        }

        /* 如果目标为目录，则创建下级 */
        let dst_file = if target.is_dir() {
            target.join(Path::new(file.file_name().unwrap()))
        } else {
            target.to_path_buf()
        };

        fs::copy(file, &dst_file).context(format!(
            "Failed to copy {} to {}",
            file.to_string_lossy(),
            dst_file.to_string_lossy()
        ))?;

        fs::set_permissions(&dst_file, file.metadata()?.permissions())
            .context("Fail to copy file permissions")?;
        Ok(dst_file.to_path_buf())
    }

    pub fn write<P, B>(file: P, content: B, mode: Option<u32>) -> Result<PathBuf>
    where
        P: AsRef<Path>,
        B: AsRef<[u8]>,
    {
        if let Some(dir) = file.as_ref().parent()
            && !dir.exists()
        {
            fs::create_dir_all(dir)
                .context(format!("Fail to create dir {}", dir.to_string_lossy()))?;
        }

        fs::write(&file, content).context(format!(
            "Fail to write file {}",
            file.as_ref().to_string_lossy()
        ))?;

        if let Some(mode) = mode {
            fs::set_permissions(&file, fs::Permissions::from_mode(mode)).context(format!(
                "Fail to set permission of {}",
                file.as_ref().to_string_lossy()
            ))?;
        }

        Ok(file.as_ref().to_path_buf())
    }
}

use crate::{
    deploy::{InstallItem, Installed},
    op::apt::AptOp,
};
use log::info;

#[derive(Debug)]
pub struct Apt {
    sw_name: String,
}

impl Apt {
    pub fn from_sw<T>(sw: T) -> Self
    where
        T: Into<String>,
    {
        Self { sw_name: sw.into() }
    }
}

impl InstallItem for Apt {
    fn check(&self) -> anyhow::Result<()> {
        info!(target: "APT", "Checking package {}", self.sw_name);
        AptOp::try_get()?
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .check(&self.sw_name)
    }

    fn install(&self) -> anyhow::Result<Installed> {
        info!(target: "APT", "Installing {}", self.sw_name);
        AptOp::try_get()?
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .install(&self.sw_name)?;

        Ok(Installed::Apt {
            name: self.sw_name.clone(),
        })
    }
}

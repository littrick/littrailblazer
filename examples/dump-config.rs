use anyhow::{Ok, Result};
use std::collections::HashMap;
use std::{fs::File, io::Write};
// use distro_pioneer::types::config::Config;
use distro_pioneer::types::config::{Config, Content, StringOr};

fn main() -> Result<()> {
    {
        let mut config = Config::default();

        // sw
        config.install.apt = Some(vec!["sw1".into(), "sw2".into()]);

        // rc
        config.install.envrc = Some(vec![
            Content::File("rc1".into()),
            Content::Raw("rc3 content".into()),
        ]);

        let mut env = HashMap::new();

        env.insert("ENVA".into(), "A".into());
        env.insert("ENVB".into(), "B".into());

        config.install.env = Some(env);

        let mut command = HashMap::new();
        command.insert("command1".into(), StringOr::String("echo command1".into()));
        command.insert(
            "command2".into(),
            StringOr::Object(Content::Raw("echo command2".into())),
        );
        command.insert(
            "command3".into(),
            StringOr::Object(Content::File("path/to/command3".into())),
        );

        config.install.command = Some(command);

        let mut alias = HashMap::new();
        alias.insert("cargo".into(), "echo cargo".into());
        config.install.alias = Some(alias);

        let toml = toml::to_string(&config)?;

        File::create("config_list.toml")?.write_all(toml.as_bytes())?;
    }
    Ok(())
}

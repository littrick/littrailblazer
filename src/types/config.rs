use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// 配置的root格式
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ConfigList {
    /// 配置列表
    pub configs: Vec<Config>,
}

/// 配置列表的格式
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    /// 基本信息
    pub infomation: Info,

    /// 安装列表
    pub install: InstallList,
}

/// 配置的基本信息
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Info {
    /// 配置名称，关于该配置的唯一识别id
    pub name: String,

    /// 该配置的描述说明
    pub description: Option<String>,

    /// 部署条件，默认为true，shell命令运行为true时，配置列表才会部署
    pub install_while: Option<String>,
}

/// 安装文件的列表
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct InstallList {
    /// 需要apt软件源安装的软件列表
    pub apt: Option<Vec<String>>,

    /// 命令别名，与`alias mycmd="echo mycmd run"`效果类似: <命令别名> <实际命令>
    pub alias: Option<HashMap<String, String>>,

    /// 额外自定义命令，可以是脚本内容或者二进制文件: <命令名> <脚本内容|二进制文件路径>
    pub command: Option<HashMap<String, StringOr<Content>>>,

    /// 环境变量
    pub env: Option<HashMap<String, String>>,

    /// 额外rc脚本，可以在里面定义函数，或者做一些初始化的操作，会在.bashrc里source
    pub envrc: Option<Vec<Content>>,

    /// 需要复制的文件，默认会安装在固定目录下，局限较大，不建议使用，优先使用以上列表
    pub files: Option<HashMap<PathBuf, StringOr<Content>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum StringOr<T> {
    String(String),
    Object(T),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Content {
    Raw(String),
    File(PathBuf),
    Url(String),
}

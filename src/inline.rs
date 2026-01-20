use serde::Deserialize;
use std::collections::BTreeMap;

use std::path::Path;

use crate::config::{
    Action, ActionType, Config, Mode, OutputConfig, Runtime, SnapshotConfig,
};

impl InlineConfig {
    /// Convert an InlineConfig into a filesystem-backed Config
    /// after files have been materialised into `root`.
    pub fn into_config(self, root: &Path) -> Config {
        Config {
            action: Some(Action {
                action_type: match self.action.language {
                    InlineLanguage::Js => ActionType::Js,
                    InlineLanguage::Python => ActionType::Python,
                },
                entry: root
                    .join(&self.action.entry)
                    .to_string_lossy()
                    .to_string(),
            }),

            fixtures: self
                .fixtures
                .into_iter()
                .map(|f| root.join(f.name).to_string_lossy().to_string())
                .collect(),

            env: self.env,

            runtime: Runtime {
                node: self.runtime.node,
                python: self.runtime.python,
            },

            snapshots: SnapshotConfig {
                enabled: self.snapshots.enabled,
                ignore: vec![],
            },

            output: OutputConfig::default(),

            budgets: None,
            assertions: Default::default(),
            assertions_file: None,

            watch: false,
            repeat: self.repeat,
            mode: Mode::Normal,
        }
    }
}


#[derive(Debug, Deserialize)]
pub struct InlineConfig {
    pub version: u32,

    pub action: InlineAction,

    #[serde(default)]
    pub fixtures: Vec<InlineFixture>,

    #[serde(default)]
    pub env: BTreeMap<String, String>,

    #[serde(default)]
    pub runtime: InlineRuntime,

    #[serde(default)]
    pub snapshots: InlineSnapshots,

    #[serde(default = "default_repeat")]
    pub repeat: u32,
}

#[derive(Debug, Deserialize)]
pub struct InlineAction {
    pub language: InlineLanguage,
    pub entry: String,
    pub source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InlineLanguage {
    Js,
    Python,
}

#[derive(Debug, Deserialize)]
pub struct InlineFixture {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct InlineRuntime {
    #[serde(default = "default_node")]
    pub node: String,

    #[serde(default = "default_python")]
    pub python: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct InlineSnapshots {
    pub enabled: bool,
}

fn default_node() -> String {
    "node".into()
}

fn default_python() -> String {
    "python".into()
}

fn default_repeat() -> u32 {
    1
}

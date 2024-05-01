use std::convert::Infallible;
use std::str::FromStr;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::serde_as;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub frontend: IndexMap<String, FrontendConfig>,
    #[serde(default)]
    pub backend: IndexMap<String, BackendConfig>,
    #[serde(default)]
    pub data: IndexMap<String, DataConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FrontendConfig {
    HttpRest(HttpRestConfig),
    HomematicCcu3(HomematicCcu3Config),
    Mqtt(MqttConfig),
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BackendConfig {
    Postgres(PostgresConfig),
}

// frontends
#[derive(Debug, Clone, Deserialize)]
pub struct HttpRestConfig {
    pub url: String,
    pub basic_auth: Option<BasicAuth>,
    pub frequency_secs: u32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct HomematicCcu3Config {
    pub url: String,
    pub basic_auth: Option<BasicAuth>,
    pub frequency_secs: u32,
    pub username: String,
    pub password: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
    pub auth: Option<MqttAuth>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MqttAuth {
    pub username: String,
    pub password: String,
}

// backends
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    #[serde(default = "default_postgres_port")]
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: Option<String>,
}

// data
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct DataConfig {
    pub frontend: FrontendRef,
    pub backend: BackendRef,
    pub persistent_every_secs: Option<u32>,
    pub clean_non_persistent_after_days: Option<u32>,
    #[serde_as(as = "IndexMap<_, serde_with::PickFirst<(serde_with::DisplayFromStr, _)>>")]
    pub values: IndexMap<String, Value>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Value {
    pub pointer: String,
    /// rebo code taking `value`-string before escaping and returning its replacement-string
    pub preprocess: Option<String>,
    /// rebo code taking `value`-string after escaping and returning its replacement-string
    pub postprocess: Option<String>,
    #[serde(default)]
    pub aggregate: Aggregate,
}
#[derive(Debug, Clone, Deserialize)]
pub enum Aggregate {
    None,
    IncrementingValueWhichMayReset,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FrontendRef {
    pub name: String,
    #[serde(flatten)]
    pub data: Option<FrontendRefData>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum FrontendRefData {
    Mqtt { mqtt_topic: String },
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum BackendRef {
    // `{ "name": "stdout" }`
    Stdout(StdoutRef),
    Postgres(PostgresRef),
}
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "name")]
pub enum StdoutRef {
    #[serde(rename = "stdout")]
    Stdout,
}
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresRef {
    pub name: String,
    pub postgres_table: String,
}



impl FromStr for Value {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Value {
            pointer: s.to_string(),
            preprocess: None,
            postprocess: None,
            aggregate: Aggregate::default(),
        })
    }
}
impl Default for Aggregate {
    fn default() -> Self {
        Aggregate::None
    }
}

fn default_mqtt_port() -> u16 { 1883 }
fn default_postgres_port() -> u16 { 5432 }

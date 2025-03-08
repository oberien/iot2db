use std::collections::HashMap;
use std::convert::Infallible;
use std::str::FromStr;
use indexmap::IndexMap;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany, formats::PreferOne};

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
    Shell(ShellConfig),
    Journald(JournaldConfig),
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
    #[serde(default = "default_mqtt_client_id")]
    pub client_id: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct MqttAuth {
    pub username: String,
    pub password: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ShellConfig {
    pub cmd: String,
    pub frequency_secs: u32,
    #[serde(default)]
    pub regex: HashMap<String, String>,
}
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct JournaldConfig {
    #[serde(default = "default_true")]
    pub system: bool,
    #[serde(default = "default_true")]
    pub current_user: bool,
    pub directory: Option<String>,
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    pub unit: Vec<String>,
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
#[derive(Debug, Clone, Deserialize)]
pub struct DataConfig {
    pub frontend: FrontendRef,
    pub backend: BackendRef,
    pub persistent_every_secs: Option<u32>,
    pub clean_non_persistent_after_days: Option<u32>,
    #[serde(flatten)]
    pub mapping: Mapping,
}
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Mapping {
    pub direct_values: Option<DirectValues>,
    #[serde_as(as = "IndexMap<_, serde_with::PickFirst<(serde_with::DisplayFromStr, _)>>")]
    pub values: IndexMap<String, Value>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DirectValues {
    /// `direct_values = all`
    All(DirectValuesAll),
    /// `direct_values = ["foo", "bar"]`
    Keys(Vec<String>),
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirectValuesAll { All }

#[derive(Debug, Clone, Deserialize)]
pub struct Value {
    #[serde(flatten)]
    pub kind: ValueKind,
    /// rebo code taking `value`-string before escaping and returning its replacement-string
    /// if there is no value, e.g. the json-pointer doesn't exist, this is _not_ executed
    pub preprocess: Option<String>,
    /// rebo code taking `value`-string after escaping and returning its replacement-string
    /// if there is no value, e.g. the json-pointer doesn't exist, this is _not_ executed
    pub postprocess: Option<String>,
    #[serde(default)]
    pub aggregate: Aggregate,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ValueKind {
    Pointer { pointer: String },
    Constant { constant_value: String },
}
#[derive(Debug, Clone, Deserialize)]
pub enum Aggregate {
    None,
    IncrementingValueWhichMayReset,
}
#[derive(Debug, Clone, Deserialize)]
pub struct FrontendRef {
    pub name: String,
    pub data_type: DataType,
    #[serde(flatten)]
    pub data: Option<FrontendRefData>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum FrontendRefData {
    Mqtt {
        mqtt_topic: String,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataType {
    /// Data is provided as a single JSON-blob
    ///
    /// Examples:
    /// * a REST API provides the result as JSON response
    /// * MQTT provides a full JSON-blob as a single message
    Wide,
    /// Data is provided with one value per message
    ///
    /// Examples:
    /// * MQTT provides one value per message / topic
    Narrow,
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
            kind: ValueKind::Pointer { pointer: s.to_string() },
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
fn default_mqtt_client_id() -> String { "iot2db".to_string() }
fn default_postgres_port() -> u16 { 5432 }
fn default_true() -> bool { true }

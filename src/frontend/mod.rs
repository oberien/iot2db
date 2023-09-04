use std::borrow::Borrow;
use std::collections::HashMap;
use std::process::exit;
use futures::stream::BoxStream;
use futures::StreamExt;
use serde_json::Value;
use crate::config;
use crate::config::{FrontendConfig, FrontendRef, FrontendRefData, HomematicCcu3Config, HttpRestConfig};
use crate::frontend::mqtt::MqttFrontend;

mod http_rest;
mod homematic_ccu3;
mod mqtt;

enum Frontend {
    HomematicCcu3(HomematicCcu3Config),
    HttpRest(HttpRestConfig),
    Mqtt(MqttFrontend),
}

pub struct Frontends {
    frontends: HashMap<String, Frontend>,
}

impl Frontends {
    pub fn new() -> Self {
        Self { frontends: HashMap::new() }
    }

    pub async fn add(&mut self, name: String, config: FrontendConfig) {
        let frontend = match config {
            FrontendConfig::HomematicCcu3(config) => Frontend::HomematicCcu3(config),
            FrontendConfig::HttpRest(config) => Frontend::HttpRest(config),
            FrontendConfig::Mqtt(config) => Frontend::Mqtt(MqttFrontend::new(&config).await),
        };
        let old = self.frontends.insert(name.clone(), frontend);
        if !old.is_none() {
            eprintln!("duplicate definition of frontend {name}");
            exit(1);
        }
    }

    pub async fn stream<T: Borrow<config::Value>>(&self, frontend_ref: FrontendRef, values: impl Iterator<Item = T>) -> BoxStream<'static, Value> {
        match self.frontends.get(&frontend_ref.name) {
            Some(Frontend::HomematicCcu3(hm)) => {
                assert_eq!(frontend_ref.data, None);
                homematic_ccu3::stream(hm.clone(), values).boxed()
            }
            Some(Frontend::HttpRest(rest)) => {
                assert_eq!(frontend_ref.data, None);
                http_rest::stream(rest.clone()).boxed()
            }
            Some(Frontend::Mqtt(mqtt)) => {
                let Some(FrontendRefData::Mqtt { mqtt_topic }) = frontend_ref.data else {
                    panic!("Usage of MQTT frontend {} requires data to provide mqtt_topic", frontend_ref.name)
                };
                mqtt.subscribe(mqtt_topic).await.boxed()
            }
            None => panic!("unknown frontend {} for data", frontend_ref.name),
        }
    }
}

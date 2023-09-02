use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use futures::StreamExt;
use crate::config::{BackendConfig, BackendRef, Config, FrontendConfig, FrontendRefData};
use rebo::{FromValue, IntoValue, ReboConfig, ReturnValue};
use crate::backend::{Backend, DataToInsert, NoopEscaper};
use crate::backend::postgres::PostgresBackend;
use crate::frontend::mqtt::MqttFrontend;

mod config;
mod frontend;
mod data;
mod backend;

#[tokio::main]
async fn main() {
    let config_path = match std::env::var("IOT2DB_CONFIG_FILE") {
        Ok(file) => PathBuf::from(file),
        Err(_) => {
            eprintln!("no config file passed via IOT2DB_CONFIG_FILE environment variable");
            return;
        }
    };
    let config_path = config_path.canonicalize()
        .expect("can't canonicalize config path");
    eprintln!("loading config file {}", config_path.display());
    let config_content = std::fs::read_to_string(config_path)
        .expect("can't read config file");
    let config: Config = toml::from_str(&config_content)
        .expect("error in config file");

    println!("{:#?}", config);

    // let cfg = match config.frontend["homematic"].clone() {
    //     FrontendConfig::HomematicCcu3(c) => c,
    //     _ => unreachable!(),
    // };
    // let data = config.data["homematic"].clone();
    // let foo = frontend::homematic_ccu3::stream(cfg, data.values.values());
    // let mapper = data::mapper(data.values.clone(), Arc::new(NoopEscaper));
    // foo.map(mapper).for_each(|value| async move { println!("{value:#?}") }).await;

    let mut pg_backends = HashMap::new();
    for (name, config) in config.backend {
        match config {
            BackendConfig::Postgres(pgconfig) => assert!(pg_backends.insert(name.clone(), PostgresBackend::new(pgconfig).await).is_none(), "duplicate definition of postgres backend {:?}", name),
        }
    }

    let mut rest_frontends = HashMap::new();
    let mut hmccu_frontends = HashMap::new();
    let mut mqtt_frontends = HashMap::new();
    for (name, config) in config.frontend {
        match config {
            FrontendConfig::HttpRest(rest) => assert!(rest_frontends.insert(name.clone(), rest).is_none(), "duplicate definition of rest frontend {:?}", name),
            FrontendConfig::HomematicCcu3(hmccu) => assert!(hmccu_frontends.insert(name.clone(), hmccu).is_none(), "duplicate definition of homematic-ccu3 frontend {:?}", name),
            FrontendConfig::Mqtt(mqtt) => assert!(mqtt_frontends.insert(name.clone(), MqttFrontend::new(&mqtt).await).is_none(), "duplicate definition of mqtt frontend {:?}", name),
        }
    }

    let mut spawn_handles = Vec::new();
    for (data_name, data) in config.data {
        // get frontend stream
        let rest = rest_frontends.get(&data.frontend.name);
        let hm = hmccu_frontends.get(&data.frontend.name);
        let mqtt = mqtt_frontends.get(&data.frontend.name);
        let stream = match (rest, hm, mqtt) {
            (Some(rest), None, None) => {
                assert_eq!(data.frontend.data, None);
                frontend::http_rest::stream(rest.clone()).boxed()
            }
            (None, Some(hm), None) => {
                assert_eq!(data.frontend.data, None);
                frontend::homematic_ccu3::stream(hm.clone(), data.values.values()).boxed()
            }
            (None, None, Some(mqtt)) => {
                let Some(FrontendRefData::Mqtt { mqtt_topic }) = data.frontend.data else {
                    panic!("Usage of MQTT frontend {} requires data {} to provide mqtt_topic", data.frontend.name, data_name)
                };
                mqtt.subscribe(mqtt_topic).await.boxed()
            }
            (None, None, None) => panic!("unknown frontend {} for data {}", data.frontend.name, data_name),
            (_, _, _) => panic!("frontend {} used by data {} defined multiple times", data.frontend.name, data_name),
        };

        // get backend sink
        let (inserter, escaper) = match data.backend {
            BackendRef::Postgres(pgref) => {
                let backend = pg_backends.get(&pgref.name)
                    .unwrap_or_else(|| panic!("unknown postgres backend {:?} for data {:?}", pgref.name, data_name));
                let inserter = backend.inserter(pgref).await;
                let escaper = backend.escaper().await;

                // periodic deletions of non-permanent data
                let inserter2 = Arc::clone(&inserter);
                if let Some(days) = data.clean_non_persistent_after_days {
                    // don't register join handle as this can just die
                    tokio::spawn(async move {
                        // once a day
                        let mut interval = tokio::time::interval(Duration::from_secs(60 * 60 * 24));
                        loop {
                            interval.tick().await;
                            inserter2.delete_old_non_persistent(days).await;
                        }
                    });
                }

                // sink for pipeline
                (inserter, escaper)
            }
        };

        // get value- / data mapper
        let mapper = data::mapper(data.values, escaper);

        // pipe everything into another
        let future = stream
            .map(mapper)
            .map(move |values| DataToInsert { escaped_values: values, persistent_every_secs: data.persistent_every_secs })
            .for_each(move |data| {
                let inserter = Arc::clone(&inserter);
                async move { inserter.insert(data).await }
            });
        let handle = tokio::spawn(future);
        spawn_handles.push(handle);
    }

    // let FrontendConfig::HttpRest(http_rest) = &config.frontend["my-rest"] else { panic!("uff") };
    // let mapper = data::mapper::<PostgresBackend>(&config.data["ahoydtu"].values);
    // let BackendConfig::Postgres(postgres) = &config.backend["my-postgres"];
    // let BackendRef::Postgres(postgres_ref) = &config.data["ahoydtu"].backend;
    // let consumer = PostgresBackend::new(postgres, postgres_ref).await;
    //
    // let stream = frontend::http_rest::stream(http_rest);
    // let mapped = stream.map(mapper);
    // let consumed = mapped.for_each(|values| consumer.consume(values));

    // let mut stream = pin!(consumed);

    // while let Some(v) = stream.next().await {
    //     println!("{v:#?}");
    // }
    // consumed.await

    futures::future::join_all(spawn_handles).await;
}

fn run_rebo<T: FromValue + IntoValue>(code: String, value: T) -> T {
    let config = ReboConfig::new().add_external_value("value".to_string(), value);
    let res = rebo::run_with_config("processing".to_string(), code, config);
    let ReturnValue::Ok(value) = res.return_value else { panic!("invalid rebo code") };
    T::from_value(value)
}

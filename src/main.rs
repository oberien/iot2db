use std::collections::HashMap;
use std::path::PathBuf;
use futures::StreamExt;
use crate::config::{BackendConfig, BackendRef, Config, FrontendConfig, FrontendRef};
use rebo::{FromValue, IntoValue, ReboConfig, ReturnValue};
use crate::backend::postgres::{PostgresBackend, PostgresEscaper};
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

    let mut pg_backends = HashMap::new();
    for (name, config) in config.backend {
        match config {
            BackendConfig::Postgres(pgconfig) => assert!(pg_backends.insert(name.clone(), PostgresBackend::new(pgconfig).await).is_none(), "duplicate definition of postgres backend {:?}", name),
        }
    }

    let mut rest_frontends = HashMap::new();
    let mut mqtt_frontends = HashMap::new();
    for (name, config) in config.frontend {
        match config {
            FrontendConfig::HttpRest(rest) => assert!(rest_frontends.insert(name.clone(), rest).is_none(), "duplicate definition of rest frontend {:?}", name),
            FrontendConfig::Mqtt(mqtt) => assert!(mqtt_frontends.insert(name.clone(), MqttFrontend::new(&mqtt).await).is_none(), "duplicate definition of mqtt frontend {:?}", name),
        }
    }

    let mut spawn_handles = Vec::new();
    for (data_name, data) in config.data {
        let stream = match data.frontend {
            FrontendRef::HttpRest { name } => {
                let frontend = rest_frontends.get(&name)
                    .unwrap_or_else(|| panic!("unknown rest frontend {:?} for data {:?}", name, data_name));
                frontend::http_rest::stream(frontend.clone()).boxed()
            },
            FrontendRef::Mqtt { name, mqtt_topic } => {
                let frontend = mqtt_frontends.get(&name)
                    .unwrap_or_else(|| panic!("unknown mqtt frontend {:?} for data {:?}", name, data_name));
                frontend.subscribe(mqtt_topic).await.boxed()
            }
        };

        let (sink, escaper) = match data.backend {
            BackendRef::Postgres(pgref) => {
                let backend = pg_backends.get(&pgref.name)
                    .unwrap_or_else(|| panic!("unknown postgres backend {:?} for data {:?}", pgref.name, data_name));
                (backend.sink(pgref), PostgresEscaper)
            }
        };

        let mapper = data::mapper(data.values, escaper);

        let handle = tokio::spawn(stream.map(mapper).map(|x| Result::<_, ()>::Ok(x)).forward(sink));
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

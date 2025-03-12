use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use futures::{future, StreamExt};
use crate::config::{BackendConfig, BackendRef, Config, DataType};
use rebo::{FromValue, IntoValue, ReboConfig, ReturnValue};
use serde_json::Value as JsonValue;
use crate::backend::{Backend, DataToInsert, Stdout};
use crate::backend::postgres::PostgresBackend;
use crate::data::{DataMapper, NarrowToWide, WideToWide};
use crate::frontend::Frontends;

mod config;
mod frontend;
mod data;
mod backend;
mod iter_json_value;

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

    let mut pg_backends = HashMap::new();
    for (name, config) in config.backend {
        match config {
            BackendConfig::Postgres(pgconfig) => assert!(pg_backends.insert(name.clone(), PostgresBackend::new(pgconfig).await).is_none(), "duplicate definition of postgres backend {:?}", name),
        }
    }

    let mut frontends = Frontends::new();
    for (name, config) in config.frontend {
        frontends.add(name, config).await;
    }

    let mut spawn_handles = Vec::new();
    for (data_name, data) in config.data {
        // get frontend stream
        let frontend_data_type = data.frontend.data_type;
        let stream = frontends.stream(data.frontend, data.mapping.values.values()).await;

        // get backend sink
        let (escaper, inserter) = match data.backend {
            BackendRef::Stdout(_) => {
                let stdout = Stdout::new(()).await;
                (stdout.escaper().await, stdout.inserter(()).await)
            }
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
                (escaper, inserter)
            }
        };

        // get value- / data mapper
        let mut mapper: Box<dyn DataMapper + Send> = match frontend_data_type {
            DataType::Wide => Box::new(WideToWide::new(data.mapping, escaper)),
            DataType::Narrow => Box::new(NarrowToWide::new(data.mapping, escaper)),
        };

        // pipe everything into another
        let future = stream
            .filter(move |value| future::ready({
                data.filter.as_ref()
                    .map(|code| filter_rebo(code.clone(), value.clone()))
                    .unwrap_or(true)
            }))
            .filter_map(move |value| future::ready(mapper.consume_value(value)))
            .map(move |values| DataToInsert { escaped_values: values, persistent_every_secs: data.persistent_every_secs })
            .for_each(move |data| {
                let inserter = Arc::clone(&inserter);
                async move { inserter.insert(data).await }
            });
        let handle = tokio::spawn(future);
        spawn_handles.push(handle);
    }

    future::join_all(spawn_handles).await;
}

fn filter_rebo(code: String, value: JsonValue) -> bool {
    let config = ReboConfig::new().add_external_value("values".to_string(), value);
    let res = rebo::run_with_config("processing".to_string(), code, config);
    let ReturnValue::Ok(value) = res.return_value else { panic!("invalid rebo code") };
    bool::from_value(value)
}

fn run_rebo<T: FromValue + IntoValue>(code: String, value: T) -> T {
    let config = ReboConfig::new().add_external_value("value".to_string(), value);
    let res = rebo::run_with_config("processing".to_string(), code, config);
    let ReturnValue::Ok(value) = res.return_value else { panic!("invalid rebo code") };
    T::from_value(value)
}

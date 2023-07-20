use std::path::PathBuf;
use std::pin::pin;
use crate::config::{BackendConfig, BackendRef, Config, FrontendConfig};
use futures::StreamExt;
use rebo::{FromValue, IntoValue, ReboConfig, ReturnValue};
use crate::backend::Backend;
use crate::backend::postgres::PostgresBackend;

mod config;
mod frontend;
mod data;
mod backend;

#[tokio::main]
async fn main() {
    let config_path = match std::env::var("IOT2DB_CONFIG_FILE") {
        Ok(file) => PathBuf::from(file),
        Err(_) => xdg::BaseDirectories::with_prefix("iot2db")
            .expect("can't init xdg_dirs")
            .find_config_file("iot2db.toml")
            .expect("no config file `iot2db.toml` found"),
    };
    let config_path = config_path.canonicalize()
        .expect("can't canonicalize config path");
    eprintln!("loading config file {}", config_path.display());
    let config_content = std::fs::read_to_string(config_path)
        .expect("can't read config file");
    let config: Config = toml::from_str(&config_content)
        .expect("error in config file");

    println!("{:#?}", config);

    let FrontendConfig::HttpRest(http_rest) = &config.frontend["my-rest"] else { panic!("uff") };
    let mapper = data::mapper::<PostgresBackend>(&config.data["ahoydtu"].values);
    let BackendConfig::Postgres(postgres) = &config.backend["my-postgres"];
    let BackendRef::Postgres(postgres_ref) = &config.data["ahoydtu"].backend;
    let consumer = PostgresBackend::new(postgres, postgres_ref).await;

    let stream = frontend::http_rest::stream(http_rest);
    let mapped = stream.map(mapper);
    let consumed = mapped.for_each(|values| consumer.consume(values));

    // let mut stream = pin!(consumed);

    // while let Some(v) = stream.next().await {
    //     println!("{v:#?}");
    // }
    consumed.await
}

fn run_rebo<T: FromValue + IntoValue>(code: String, value: T) -> T {
    let config = ReboConfig::new().add_external_value("value".to_string(), value);
    let res = rebo::run_with_config("processing".to_string(), code, config);
    let ReturnValue::Ok(value) = res.return_value else { panic!("invalid rebo code") };
    T::from_value(value)
}

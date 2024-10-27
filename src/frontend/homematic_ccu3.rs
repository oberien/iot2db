use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use futures::{future, FutureExt, Stream, StreamExt, TryFutureExt};
use futures::stream::FuturesUnordered;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use crate::config;
use crate::config::{BasicAuth, HomematicCcu3Config, ValueKind};

#[derive(Default)]
struct ParametersetToLoad {
    load_values: bool,
    load_master: bool,
}

pub fn stream<T: Borrow<config::Value>>(config: HomematicCcu3Config, accessed_values: impl Iterator<Item = T>) -> impl Stream<Item = Value> + 'static {
    // check which device's channel's parameterSets to load
    let mut parametersets_to_load: HashMap<(String, usize), ParametersetToLoad> = HashMap::new();
    for value in accessed_values {
        let pointer = match &value.borrow().kind {
            ValueKind::Pointer { pointer } => pointer,
            ValueKind::Constant { .. } => continue,
        };
        let mut parts = pointer.split('/').skip(1).map(|x| x.replace("~1", "/").replace("~0", "~"));
        let Some(device_name) = parts.next() else { continue };
        let Some("channels") = parts.next().as_deref() else { continue };
        let Some(channel) = parts.next().and_then(|c| c.parse().ok()) else { continue };
        let entry = parametersets_to_load.entry((device_name, channel)).or_default();
        match parts.next().as_deref() {
            Some("values") => entry.load_values = true,
            Some("master") => entry.load_master = true,
            _ => continue,
        }
    }

    let client = Arc::new(JsonRpc::new(&config));
    let config = Arc::new(config);
    let parametersets_to_load = Arc::new(parametersets_to_load);
    futures::stream::unfold(0, move |mut iteration| {
        let client = Arc::clone(&client);
        let config = Arc::clone(&config);
        let parametersets_to_load = Arc::clone(&parametersets_to_load);
        async move {
            'repeat: loop {
                if iteration != 0 {
                    tokio::time::sleep(Duration::from_secs(config.frequency_secs as u64)).await;
                }
                iteration += 1;

                macro_rules! jsonrpc {
                    ($method:literal, $params:expr) => {
                        match client.jsonrpc($method, $params).await {
                            Ok(val) => val,
                            Err(e) => {
                                eprintln!("homematic-ccu3: Error executing {}: {e:?}", $method);
                                continue
                            }
                        }
                    }
                }

                // login
                let session_id: String = jsonrpc!("Session.login", json!({
                    "username": &config.username,
                    "password": &config.password,
                }));

                // get device list
                let mut devices: Vec<Value> = jsonrpc!("Device.listAllDetail", json!({
                    "_session_id_": session_id,
                }));

                // fetch relevant parameterSets
                let mut channel_futures = FuturesUnordered::new();
                for device in &mut devices {
                    let interface = device["interface"].as_str().unwrap().to_string();
                    let name = device["name"].as_str().unwrap().to_string();
                    let client = Arc::new(&client);
                    let session_id = session_id.clone();

                    for (i, channel) in device["channels"].as_array_mut().unwrap().into_iter().enumerate() {
                        let channel = channel.as_object_mut().unwrap();
                        channel.insert("values".to_string(), json!({"NOT_LOADED": "this object hasn't been loaded - access it to load it"}));

                        let Some(parametersets_to_load) = parametersets_to_load.get(&(name.clone(), i)) else {
                            channel.insert("values".to_string(), json!({"NOT_LOADED": "this object hasn't been loaded - access it to load it"}));
                            channel.insert("master".to_string(), json!({"NOT_LOADED": "this object hasn't been loaded - access it to load it"}));
                            continue
                        };
                        let values = if parametersets_to_load.load_values {
                            client.jsonrpc::<_, Value>("Interface.getParamset", json!({
                                "_session_id_": session_id,
                                "address": channel["address"],
                                "interface": interface,
                                "paramsetKey": "VALUES",
                            })).map_err(|e| format!("{e:?}")).map(Some).boxed()
                        } else {
                            channel.insert("values".to_string(), json!({"NOT_LOADED": "this object hasn't been loaded - access it to load it"}));
                            futures::future::ready(None).boxed()
                        };
                        let master = if parametersets_to_load.load_master {
                            client.jsonrpc::<_, Value>("Interface.getParamset", json!({
                                "_session_id_": session_id,
                                "address": channel["address"],
                                "interface": interface,
                                "paramsetKey": "MASTER",
                            })).map_err(|e| format!("{e:?}")).map(Some).boxed()
                        } else {
                            channel.insert("master".to_string(), json!({"NOT_LOADED": "this object hasn't been loaded - access it to load it"}));
                            futures::future::ready(None).boxed()
                        };

                        channel_futures.push(future::join(master, values)
                            .map(|(m, v)| {
                                match m {
                                    Some(Ok(m)) => { channel.insert("master".to_string(), m); },
                                    Some(Err(e)) => {
                                        eprintln!("homematic-ccu3: Error executing Interface.getParamset MASTER: {e}");
                                        channel.insert("master".to_string(), json!({ "NOT_LOADED": "Error loading" }));
                                        return Err(())
                                    },
                                    None => (),
                                }
                                match v {
                                    Some(Ok(v)) => { channel.insert("values".to_string(), v); },
                                    Some(Err(e)) => {
                                        eprintln!("homematic-ccu3: Error executing Interface.getParamset VALUES: {e}");
                                        channel.insert("master".to_string(), json!({ "NOT_LOADED": "Error loading" }));
                                        return Err(())
                                    },
                                    None => (),
                                }
                                Ok(())
                            }));
                    }
                }

                while let Some(res) = channel_futures.next().await {
                    match res {
                        Ok(()) => (),
                        Err(()) => {
                            eprintln!("homematic-ccu3: error fetching values - retry");
                            continue 'repeat
                        },
                    }
                }
                drop(channel_futures);

                let devices = devices.into_iter()
                    .map(|d| (d["name"].as_str().unwrap().to_owned(), d))
                    .collect();

                // logout
                let _: Value = jsonrpc!("Session.logout", json!({
                    "_session_id_": session_id,
                }));

                let res = Value::Object(devices);
                break Some((res, iteration))
            }
        }
    })
}

struct JsonRpc {
    client: Client,
    basic_auth: Option<BasicAuth>,
    url: String,
}

#[derive(Serialize)]
struct JsonRpcRequest<'a, T: Serialize> {
    version: &'a str,
    method: &'a str,
    params: T,
}
#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    version: String,
    result: Option<T>,
    error: Option<Value>,
}

impl JsonRpc {
    fn new(config: &HomematicCcu3Config) -> Self {
        Self {
            client: reqwest::ClientBuilder::new()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("can't build reqwest client"),
            basic_auth: config.basic_auth.clone(),
            url: format!("{}/api/homematic.cgi", config.url),
        }
    }

    async fn jsonrpc<Req: Serialize, Res: DeserializeOwned>(&self, method: &str, params: Req) -> Result<Res, Box<dyn Error>> {
        let mut req = self.client.post(&self.url);
        if let Some(auth) = &self.basic_auth {
            req = req.basic_auth(&auth.username, auth.password.as_ref());
        }
        let res = req.json(&JsonRpcRequest {
            version: "1.1",
            method,
            params,
        }).send().await?;
        let text = res.text().await?;
        let json: JsonRpcResponse<Res> = serde_json::from_str(&text)?;
        if json.version != "1.1" {
            return Err("JSON-RPC version is not 1.1".into());
        }
        if let Some(error) = json.error {
            return Err(format!("JSON-RPC Response Error: {error}").into());
        }
        if let Some(result) = json.result {
            return Ok(result);
        }
        Err("JSON-RPC got neither error nor value:".into())

    }
}



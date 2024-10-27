use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use futures::Stream;
use regex::Regex;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde_json::{json, Value};
use tokio::sync::broadcast::{self, Sender};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};
use crate::config::MqttConfig;

pub struct MqttFrontend {
    client: AsyncClient,
    receivers: Arc<StdMutex<Vec<(Regex, Sender<Value>)>>>,
}

impl MqttFrontend {
    pub async fn new(config: &MqttConfig) -> Self {
        let mut options = MqttOptions::new(&config.client_id, &config.host, config.port);
        options.set_keep_alive(Duration::from_secs(10));
        if let Some(auth) = &config.auth {
            options.set_credentials(&auth.username, &auth.password);
        }
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        let receivers: Arc<StdMutex<Vec<(Regex, Sender<Value>)>>> = Arc::new(StdMutex::new(Vec::new()));

        let receivers2 = Arc::clone(&receivers);
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::Publish(p))) => {
                        let value: Value = match serde_json::from_slice(&p.payload) {
                            Ok(value) => value,
                            // if it's not JSON, interpret it as simple String
                            Err(e) => Value::String(String::from_utf8_lossy(&p.payload).into_owned()),
                        };
                        let value = json!({ &p.topic: value });
                        let receivers = receivers2.lock().unwrap();
                        let senders = receivers.iter()
                            .filter_map(|(regex, sender)| regex.is_match(&p.topic).then_some(sender));
                        let mut sent = false;
                        for sender in senders {
                            sender.send(value.clone()).unwrap();
                            sent = true;
                        }
                        if !sent {
                            eprintln!("got message for topic {:?} but can't find any subscriber", p.topic);
                        }
                    },
                    Err(x) => eprintln!("error in mqtt: {x:?}"),
                    _ => (),
                }
            }
        });
        MqttFrontend { client, receivers }
    }

    pub async fn stream(&self, topic: String) -> impl Stream<Item = Value> {
        // convert topic into a regex
        let pattern = regex::escape(&topic);
        // topic wildcard `+` matches everything in a single-level
        let mut pattern = pattern.replace("\\+", "[^/]*");
        // wildcard `#` matches everything in all levels, but must be at the end
        if pattern.ends_with("\\#") {
            assert_eq!(pattern.pop(), Some('#'));
            assert_eq!(pattern.pop(), Some('\\'));
            pattern.push_str(".*");
        };
        // pattern must match the full string
        let pattern = format!("^{pattern}$");
        let rx = {
            let mut receivers = self.receivers.lock().unwrap();
            let receiver = receivers.iter().find(|(regex, _)| regex.as_str() == &pattern);
            match receiver {
                Some((_topic, sender)) => sender.subscribe(),
                None => {
                    receivers.push((Regex::new(&pattern).unwrap(), broadcast::channel(200).0));
                    receivers.last().unwrap().1.subscribe()
                },
            }
        };
        self.client.subscribe(&topic, QoS::AtMostOnce).await.unwrap();
        println!("subscribed to `{topic}` using regex `{pattern}`");
        BroadcastStream::new(rx)
            .filter_map(move |val| match val {
                Ok(val) => Some(val),
                Err(BroadcastStreamRecvError::Lagged(num)) => {
                    eprintln!("mqtt receiver for {topic} lagged by {num}");
                    None
                }
            })
    }
}

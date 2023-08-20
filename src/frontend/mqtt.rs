use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use futures::Stream;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::config::MqttConfig;

pub struct MqttFrontend {
    client: AsyncClient,
    receivers: Arc<StdMutex<HashMap<String, Vec<UnboundedSender<Value>>>>>,
}

impl MqttFrontend {
    pub async fn new(config: &MqttConfig) -> Self {
        let mut options = MqttOptions::new("iot2db", &config.host, config.port);
        options.set_keep_alive(Duration::from_secs(10));
        if let Some(auth) = &config.auth {
            options.set_credentials(&auth.username, &auth.password);
        }
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        let receivers: Arc<StdMutex<HashMap<String, Vec<UnboundedSender<Value>>>>> = Arc::new(StdMutex::new(HashMap::new()));

        let receivers2 = Arc::clone(&receivers);
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Incoming::Publish(p))) => {
                        let value: Value = match serde_json::from_slice(&p.payload) {
                            Ok(value) => value,
                            Err(e) => {
                                eprintln!("error decoding mqtt payload to json: {:?} ({:?})", e, p.payload);
                                continue
                            }
                        };
                        match receivers2.lock().unwrap().get(&p.topic) {
                            Some(senders) => for sender in senders {
                                sender.send(value.clone()).unwrap();
                            },
                            None => {
                                eprintln!("got message for topic {:?} but can't find any subscriber", p.topic);
                                continue
                            }
                        }
                    },
                    _ => (),
                }
            }
        });
        MqttFrontend { client, receivers }
    }

    pub async fn subscribe(&self, topic: String) -> impl Stream<Item = Value> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.receivers.lock().unwrap().entry(topic.clone()).or_default().push(tx);
        self.client.subscribe(&topic, QoS::AtMostOnce).await.unwrap();
        UnboundedReceiverStream::new(rx)
    }
}

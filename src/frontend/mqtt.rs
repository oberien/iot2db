use std::collections::HashMap;
use std::time::Duration;
use futures::Stream;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;
use tokio_stream::wrappers::UnboundedReceiverStream;
use crate::config::MqttConfig;

pub struct MqttFrontend {
    client: AsyncClient,
    eventloop_tx: UnboundedSender<NewSubscriber>,
}

impl MqttFrontend {
    pub async fn new(config: &MqttConfig) -> Self {
        let mut options = MqttOptions::new("iot2db", &config.host, config.port);
        options.set_keep_alive(Duration::from_secs(10));
        if let Some(auth) = &config.auth {
            options.set_credentials(&auth.username, &auth.password);
        }
        let (client, mut eventloop) = AsyncClient::new(options, 10);
        let (eventloop_tx, mut eventloop_rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut receivers: HashMap<String, Vec<UnboundedSender<Value>>> = HashMap::new();
            loop {
                tokio::select! {
                    Some(NewSubscriber { topic, event_sender }) = eventloop_rx.recv() => {
                        receivers.entry(topic).or_default().push(event_sender);
                    },
                    Ok(event) = eventloop.poll() => {
                        match event {
                            Event::Incoming(Incoming::Publish(p)) => {
                                let value: Value = match serde_json::from_slice(&p.payload) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        eprintln!("error decoding mqtt payload to json: {:?} ({:?})", e, p.payload);
                                        continue
                                    }
                                };
                                match receivers.get(&p.topic) {
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
                    },
                }
            }
        });
        MqttFrontend { client, eventloop_tx }
    }

    pub async fn subscribe(&self, topic: String) -> impl Stream<Item = Value> {
        self.client.subscribe(&topic, QoS::AtMostOnce).await.unwrap();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        self.eventloop_tx.send(NewSubscriber { topic, event_sender: tx }).unwrap();
        UnboundedReceiverStream::new(rx)
    }
}

struct NewSubscriber {
    topic: String,
    event_sender: UnboundedSender<Value>,
}
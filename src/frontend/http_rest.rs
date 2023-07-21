use std::time::Duration;
use futures::Stream;
use serde_json::Value;
use crate::config::HttpRestConfig;

pub fn stream(config: HttpRestConfig) -> impl Stream<Item = Value> + 'static {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("can't build reqwest client");
    futures::stream::unfold(0, move |iteration| {
        let client = client.clone();
        let config = config.clone();
        async move {
            loop {
                if iteration != 0 {
                    tokio::time::sleep(Duration::from_secs(config.frequency_secs as u64)).await;
                }
                let iteration = iteration + 1;
                let mut req = client.get(&config.url);
                if let Some(auth) = &config.basic_auth {
                    req = req.basic_auth(&auth.username, auth.password.as_ref());
                }
                let res = req.send().await;
                let res = match res {
                    Ok(res) => res.json().await,
                    Err(e) => {
                        eprintln!("error performing request to {:?}: {:?}", config.url, e);
                        continue
                    }
                };
                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        eprintln!("error converting response from {:?} to json: {:?}", config.url, e);
                        continue
                    }
                };

                break Some((res, iteration))
            }
        }
    })
}
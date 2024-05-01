use std::collections::HashMap;
use std::time::Duration;
use futures::Stream;
use regex::Regex;
use serde_json::{Map, Value};
use tokio::process::Command;
use crate::config::ShellConfig;

pub fn stream(config: &ShellConfig) -> impl Stream<Item = Value> + 'static {
    let config = config.clone();
    futures::stream::unfold(0, move |mut iteration| {
        let config = config.clone();
        let args = shlex::split(&config.cmd)
            .expect("invalid command string for shell frontend");
        let program = args.first()
            .expect("command string for shell frontend doesn't contain a program");
        let args = &args[1..];
        let mut command = Command::new(program);
        command.args(args);
        let regexes: HashMap<_, _> = config.regex.iter()
            .map(|(name, regex)| (name.clone(), Regex::new(regex).unwrap_or_else(|e| panic!("invalid regex {}: {e:?}", regex))))
            .collect();
        async move {
            loop {
                if iteration != 0 {
                    tokio::time::sleep(Duration::from_secs(config.frequency_secs as u64)).await;
                }
                iteration += 1;
                let output = command.output().await;
                let output = match output {
                    Ok(output) => output,
                    Err(e) => {
                        eprintln!("error executing `{}`: {:?}", config.cmd, e);
                        continue
                    }
                }.stdout;
                let output = match String::from_utf8(output) {
                    Ok(output) => output,
                    Err(e) => {
                        eprintln!("output of shell command `{}` was not valid utf-8: {e:?}", config.cmd);
                        continue
                    }
                };
                // convert to json
                let json = match config.regex.is_empty() {
                    true => match serde_json::from_str(&output) {
                        Ok(json) => json,
                        Err(e) => {
                            eprintln!("error parsing json from command `{}`: {:?}", config.cmd, e);
                            continue
                        }
                    }
                    false => {
                        let mut map = Map::new();
                        for (name, regex) in &regexes {
                            let val = match regex.captures(&output) {
                                Some(val) => val,
                                None => {
                                    eprintln!("regex {regex} doesn't match for key {name} in output of shell command `{}`", config.cmd);
                                    continue
                                }
                            };
                            let val = match val.get(1) {
                                Some(val) => val.as_str(),
                                None => {
                                    eprintln!("no capture group found for key {name} in output of shell command `{}`", config.cmd);
                                    continue
                                }
                            };
                            match map.insert(name.clone(), Value::from(val)) {
                                None => (),
                                Some(_) => eprintln!("duplicate regex key {name:?} for shell command `{}`", config.cmd),
                            }
                        }
                        Value::Object(map)
                    }
                };

                break Some((json, iteration))
            }
        }
    })
}
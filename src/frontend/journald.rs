use std::thread;
use std::time::UNIX_EPOCH;
use futures::Stream;
use serde_json::Value;
use systemd::journal::{OpenDirectoryOptions, OpenOptions};
use tokio_stream::wrappers::ReceiverStream;
use crate::config::JournaldConfig;

pub fn stream(config: JournaldConfig) -> impl Stream<Item = Value> + 'static {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    thread::spawn(move || {
        let mut journal = match config.directory {
            Some(directory) => OpenDirectoryOptions::default()
                .system(config.system)
                .current_user(config.current_user)
                .open_directory(directory)
                .expect("can't open systemd-journal directory"),
            None => OpenOptions::default()
                .system(config.system)
                .current_user(config.current_user)
                .open()
                .expect("can't open systemd-journal"),
        };
        for unit in config.unit {
            // copy behaviour of journalctl's `add_matches_for_unit_full`
            // https://github.com/systemd/systemd/blob/main/src/shared/logs-show.c#L1664
            journal.match_add("_SYSTEMD_UNIT", unit.clone()).unwrap();
            journal.match_or().unwrap();
            journal.match_add("_PID", "1").unwrap();
            journal.match_add("UNIT", unit.clone()).unwrap();
            journal.match_or().unwrap();
            journal.match_add("_UID", "0").unwrap();
            journal.match_add("OBJECT_SYSTEMD_UNIT", unit).unwrap();
            journal.match_or().unwrap();
        }
        // this seeks after the tail, where nothing new will ever be
        journal.seek_tail().unwrap();
        // thus, we need to go back to the tail where new entries will be added
        journal.previous().unwrap();
        loop {
            loop {
                match journal.next_entry() {
                    Ok(Some(mut record)) => {
                        let unit1 = record.get("_UID").filter(|&uid| uid == "0")
                            .and(record.get("OBJECT_SYSTEMD_UNIT"));
                        let unit2 = record.get("_PID").filter(|&pid| pid == "1")
                            .and(record.get("UNIT"));
                        let unit3 = record.get("_SYSTEMD_UNIT");
                        if let Some(unit) = unit1.or(unit2).or(unit3) {
                            record.insert("__TARGET_UNIT".to_owned(), unit.clone());
                        }

                        let ts = journal.timestamp().unwrap()
                            .duration_since(UNIX_EPOCH).unwrap().as_secs();
                        record.insert("__TIMESTAMP".to_owned(), ts.to_string());

                        tx.blocking_send(Value::Object(record.into_iter().map(|(key, value)| (key, Value::String(value))).collect())).expect("can't blocking send journal to channel")
                    },
                    Ok(None) => break,
                    Err(e) => panic!("error reading journald entries: {e:?}"),
                }
            }

            journal.wait(None).unwrap();
        }
    });

    ReceiverStream::new(rx)
}
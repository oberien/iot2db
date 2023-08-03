use std::sync::Arc;
use indexmap::IndexMap;
use postgres_protocol::escape::{escape_identifier, escape_literal};
use tokio::sync::mpsc::UnboundedSender;
use tokio_postgres::{Client, Config, NoTls};
use crate::backend::{BackendInserter, DataToInsert, BackendEscaper, Backend};
use crate::config::{PostgresConfig, PostgresRef};

pub struct PostgresBackend {
    insert_tx: UnboundedSender<ClientCommand>,
}

struct PostgresInserter {
    insert_tx: UnboundedSender<ClientCommand>,
    table: String,
}

struct PostgresEscaper;
impl BackendEscaper for PostgresEscaper {
    fn escape_value(&self, value: String) -> String {
        escape_literal(&value)
    }
}

enum ClientCommand {
    InsertData(InsertData),
    DeleteOldNonPersistent(DeleteOldNonPersistent),
}
struct InsertData {
    table: String,
    escaped_values: IndexMap<String, String>,
    persistent_every_secs: Option<u32>,
}
struct DeleteOldNonPersistent {
    table: String,
    delete_older_than_days: u32,
}

#[async_trait::async_trait]
impl Backend for PostgresBackend {
    type Config = PostgresConfig;
    type Ref = PostgresRef;

    async fn new(config: PostgresConfig) -> Self {
        let mut pgcfg = Config::new();
        pgcfg.host(&config.host)
            .port(config.port)
            .dbname(&config.database)
            .user(&config.username);
        if let Some(password) = &config.password {
            pgcfg.password(password);
        }
        let (client, connection) = pgcfg.connect(NoTls).await
            .expect("can't open postgres connection");
        tokio::spawn(connection);
        let (insert_tx, mut insert_rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(command) = insert_rx.recv().await {
                match command {
                    ClientCommand::InsertData(insert_data) => insert(&client, insert_data).await,
                    ClientCommand::DeleteOldNonPersistent(delete) => delete_old_non_persistent(&client, delete).await,
                }
            }
        });
        PostgresBackend { insert_tx }
    }

    async fn escaper(&self) -> Arc<dyn BackendEscaper + Send + Sync + 'static> {
        Arc::new(PostgresEscaper)
    }

    async fn inserter(&self, pgref: PostgresRef) -> Arc<dyn BackendInserter + Send + Sync + 'static> {
        Arc::new(PostgresInserter {
            insert_tx: self.insert_tx.clone(),
            table: pgref.postgres_table,
        })
    }
}

#[async_trait::async_trait]
impl BackendInserter for PostgresInserter {
    async fn insert(&self, data: DataToInsert) {
        self.insert_tx.send(ClientCommand::InsertData(InsertData {
            table: self.table.clone(),
            escaped_values: data.escaped_values,
            persistent_every_secs: data.persistent_every_secs,
        })).unwrap();
    }

    async fn delete_old_non_persistent(&self, delete_older_than_days: u32) {
        self.insert_tx.send(ClientCommand::DeleteOldNonPersistent(DeleteOldNonPersistent {
            table: self.table.clone(),
            delete_older_than_days,
        })).unwrap();
    }
}

async fn delete_old_non_persistent(client: &Client, DeleteOldNonPersistent { table, delete_older_than_days }: DeleteOldNonPersistent) {
    let escaped_table = escape_identifier(&table);
    let query = format!("DELETE FROM {escaped_table} WHERE persistent = false AND timestamp < (NOW() - INTERVAL '{delete_older_than_days} DAYS')");
    eprintln!("{query}");
    client.execute(&query, &[]).await
        .expect("can't delete old non-persistent data");
}
async fn insert(client: &Client, InsertData { table, escaped_values, persistent_every_secs }: InsertData) {
    let escaped_table = escape_identifier(&table);
    let mut fmt = format!("INSERT INTO {} (", escaped_table);
    if persistent_every_secs.is_some() {
        fmt.push_str("persistent,");
    }
    for column in escaped_values.keys() {
        fmt.push_str(&escape_identifier(column));
        fmt.push(',');
    }
    assert_eq!(fmt.pop(), Some(','));

    fmt.push_str(") VALUES (");
    if let Some(persistent_every_secs) = persistent_every_secs {
        let current_timestamp = &escaped_values["timestamp"];
        fmt.push_str(&format!(
            "(SELECT max(\"timestamp\") + INTERVAL '{persistent_every_secs} SECONDS' <= {current_timestamp} FROM {escaped_table} where persistent),"
        ));
    }
    for value in escaped_values.values() {
        fmt.push_str(value);
        fmt.push(',');
    }
    assert_eq!(fmt.pop(), Some(','));
    fmt.push_str(") ON CONFLICT DO NOTHING");

    eprintln!("{fmt}");
    client.execute(&fmt, &[]).await
        .expect("cannot insert into postgres");
}

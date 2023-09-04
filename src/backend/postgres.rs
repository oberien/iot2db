use std::sync::Arc;
use indexmap::IndexMap;
use postgres_protocol::escape::{escape_identifier, escape_literal};
use tokio::sync::Mutex as AsyncMutex;
use tokio_postgres::{Client, Config, NoTls};
use crate::backend::{BackendInserter, DataToInsert, BackendEscaper, Backend};
use crate::config::{PostgresConfig, PostgresRef};

pub struct PostgresBackend {
    client: Arc<AsyncMutex<Client>>,
}

struct PostgresInserter {
    client: Arc<AsyncMutex<Client>>,
    table: String,
}

struct PostgresEscaper;
impl BackendEscaper for PostgresEscaper {
    fn escape_value(&self, value: String) -> String {
        escape_literal(&value)
    }
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
        let client = Arc::new(AsyncMutex::new(client));
        PostgresBackend { client }
    }

    async fn escaper(&self) -> Arc<dyn BackendEscaper + Send + Sync + 'static> {
        Arc::new(PostgresEscaper)
    }

    async fn inserter(&self, pgref: PostgresRef) -> Arc<dyn BackendInserter + Send + Sync + 'static> {
        Arc::new(PostgresInserter {
            client: Arc::clone(&self.client),
            table: pgref.postgres_table,
        })
    }
}

#[async_trait::async_trait]
impl BackendInserter for PostgresInserter {
    async fn insert(&self, data: DataToInsert) {
        insert(&*self.client.lock().await,
            &self.table,
            &data.escaped_values,
            data.persistent_every_secs,
        ).await;
    }

    async fn delete_old_non_persistent(&self, delete_older_than_days: u32) {
        delete_old_non_persistent(&*self.client.lock().await,
            &self.table.clone(),
            delete_older_than_days,
        ).await;
    }
}

async fn delete_old_non_persistent(client: &Client, table: &String, delete_older_than_days: u32) {
    let escaped_table = escape_identifier(&table);
    let query = format!("DELETE FROM {escaped_table} WHERE persistent = false AND timestamp < (NOW() - INTERVAL '{delete_older_than_days} DAYS')");
    eprintln!("{query}");
    client.execute(&query, &[]).await
        .expect("can't delete old non-persistent data");
}
async fn insert(
    client: &Client,
    table: &str,
    escaped_values: &IndexMap<String, String>,
    persistent_every_secs: Option<u32>
) {
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
            "(SELECT COALESCE(max(\"timestamp\") + INTERVAL '{persistent_every_secs} SECONDS' <= {current_timestamp}, true) FROM {escaped_table} where persistent),"
        ));
    }
    for value in escaped_values.values() {
        fmt.push_str(value);
        fmt.push(',');
    }
    assert_eq!(fmt.pop(), Some(','));
    fmt.push_str(") ON CONFLICT DO NOTHING");

    eprintln!("{fmt}");
    match client.execute(&fmt, &[]).await {
        Ok(_) => (),
        Err(e) => eprintln!("cannot insert into postgres: {e}"),
    }
}

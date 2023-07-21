use std::collections::HashMap;
use futures::Sink;
use postgres_protocol::escape::{escape_identifier, escape_literal};
use tokio::sync::mpsc::UnboundedSender;
use tokio_postgres::{Client, Config, NoTls};
use crate::backend::Escaper;
use crate::config::{PostgresConfig, PostgresRef};

pub struct PostgresBackend {
    insert_tx: UnboundedSender<Insert>,
}

impl PostgresBackend {
    pub async fn new(config: PostgresConfig) -> Self {
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
            while let Some(Insert { table, escaped_values }) = insert_rx.recv().await {
                insert(&client, &table, &escaped_values).await;
            }
        });
        PostgresBackend { insert_tx }
    }

    pub fn sink(&self, pgref: PostgresRef) -> impl Sink<HashMap<String, String>, Error = ()> + Send + 'static {
        let insert_tx = self.insert_tx.clone();
        let table = pgref.postgres_table.clone();
        futures::sink::unfold((), move |(), escaped_values| {
            let insert_tx = insert_tx.clone();
            let table = table.clone();
            async move {
                insert_tx.send(Insert {
                    table,
                    escaped_values,
                }).unwrap();
                Ok(())
            }
        })
    }
}

pub struct PostgresEscaper;
impl Escaper for PostgresEscaper {
    fn escape_value(&self, value: String) -> String {
        escape_literal(&value)
    }
}

async fn insert(client: &Client, table: &str, escaped_values: &HashMap<String, String>) {
    let mut fmt = format!("INSERT INTO {} (", escape_identifier(table));
    for column in escaped_values.keys() {
        fmt.push_str(&escape_identifier(column));
        fmt.push(',');
    }
    assert_eq!(fmt.pop(), Some(','));

    fmt.push_str(") VALUES (");
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

struct Insert {
    table: String,
    escaped_values: HashMap<String, String>,
}

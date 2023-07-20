use std::collections::HashMap;
use postgres_protocol::escape::{escape_identifier, escape_literal};
use tokio_postgres::{Client, Config, NoTls};
use crate::config::PostgresConfig;

pub struct Consumer {
    client: Client,
    table: String,
}

impl Consumer {
    pub async fn consume(&self, values: HashMap<&str, String>) {
        let mut fmt = format!("INSERT INTO {} (", escape_identifier(&self.table));
        for column in values.keys() {
            fmt.push_str(&escape_identifier(column));
            fmt.push(',');
        }
        assert_eq!(fmt.pop(), Some(','));

        fmt.push_str(") VALUES (");
        for value in values.values() {
            fmt.push_str(&escape_literal(value));
            fmt.push(',');
        }
        assert_eq!(fmt.pop(), Some(','));
        fmt.push_str(") ON CONFLICT DO NOTHING");

        eprintln!("{fmt}");

        self.client.execute(&fmt, &[]).await
            .expect("cannot insert into postgres");
    }
}

pub async fn consumer(config: &PostgresConfig, table: String) -> Consumer {
    assert!(!table.contains('"'), "table name {:?} must not contain `\"`", table);
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
    Consumer { client, table }
}
use std::collections::HashMap;
use postgres_protocol::escape::{escape_identifier, escape_literal};
use tokio_postgres::{Client, Config, NoTls};
use crate::backend::Backend;
use crate::config::{PostgresConfig, PostgresRef};

pub struct PostgresBackend {
    client: Client,
    table: String,
}

#[async_trait::async_trait]
impl Backend for PostgresBackend {
    type Config = PostgresConfig;
    type Ref = PostgresRef;

    async fn new(config: &Self::Config, r: &Self::Ref) -> Self {
        let table = r.postgres_table.clone();
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
        PostgresBackend { client, table }
    }

    async fn consume(&self, escaped_values: HashMap<&str, String>) {
        let mut fmt = format!("INSERT INTO {} (", escape_identifier(&self.table));
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

        self.client.execute(&fmt, &[]).await
            .expect("cannot insert into postgres");
    }

    fn escape_value(value: String) -> String {
        escape_literal(&value)
    }
}

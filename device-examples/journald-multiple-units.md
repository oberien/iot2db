# journald / journalctl / systemd-log

Frontend | Backend | Table Layout
--- | --- | ---
journald (wide) | postgres | narrow-ish

See [journald-single-unit](journald-single-unit.md) for more information.

**WARNING:** If you don't specify a unit / list of units and don't have a filter to filter
out iot2db, this will result in an infinite loop adding infinite entries to the db.

## References

## Setup of Postgres

```sql
-- Create User
CREATE USER journald;
-- Create Database
CREATE DATABASE journald OWNER journald;
REVOKE CONNECT ON DATABASE journald FROM PUBLIC;
-- connect to db
\c journald
-- Create Tables
SET ROLE journald;
CREATE TABLE IF NOT EXISTS logs (
    timestamp timestamp with time zone NOT NULL,
    data jsonb NOT NULL
);
CREATE INDEX ON logs (timestamp);
CREATE INDEX ON logs (timestamp, (data->>'__TARGET_UNIT'));
```

## Configuration of iot2db

```toml
[frontend.journald]
type = "journald"
#directory = "/var/log/journal"
system = true
current_user = true

[backend.postgres-journald]
type = "postgres"
host = "localhost"
#port = 5432
database = "journald"
username = "journald"
#password = ""

[data.journal]
frontend.name = "journal"
frontend.data_type = "wide"
backend.name = "postgres-journald"
backend.postgres_table = "journald"
filter = """
    values.unwrap_object().get("__TARGET_UNIT") != Option::Some(JsonValue::String("iot2db.service"))
"""
values.timestamp = { constant_value = "", postprocess = '"CURRENT_TIMESTAMP"' }
values.data = ""
```

## Example journald data

See [journald-single-unit](journald-single-unit.md).

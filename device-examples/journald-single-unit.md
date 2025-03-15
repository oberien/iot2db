# journald / journalctl / systemd-log

Frontend | Backend | Table Layout
--- | --- | ---
journald (wide) | postgres | wide

Store systemd log of a single service in the database to use for e.g. accessing
them via Grafana or creating alerts.

* Supports both standard system and user log
* Supports directory-based logs, e.g. from `systemd-journal-remote`.
* See available values (may differ for each unit) via `journalctl -eu <unit> -o json-pretty`.
* If services don't use structured logging, the only relevant field is `MESSAGE`.

**WARNING:** Don't use this to log iot2db's output to the db. Otherwise, this
will result in an infinite loop adding infinite entries to the db.

## Fields added by iot2db

* `__TARGET_UNIT`: contains the name of the unit the message is for / from
* `__TIMESTAMP`: unix timestamp (seconds) the record

## Setup of Postgres

```sql
-- Create User
CREATE USER journald_foo;
-- Create Database
CREATE DATABASE journald_foo OWNER journald_foo;
REVOKE CONNECT ON DATABASE journald_foo FROM PUBLIC;
-- connect to db
\c journald_foo
-- Create Tables
SET ROLE journald_foo;
CREATE TABLE IF NOT EXISTS logs (
    timestamp timestamp with time zone NOT NULL,
    message text NOT NULL,
    PRIMARY KEY (timestamp)
);
```

## Configuration of iot2db

```toml
[frontend.journald-foo]
type = "journald"
#directory = "/var/log/journal"
system = true
current_user = true
unit = "foo.service"

[backend.postgres-journald-foo]
type = "postgres"
host = "localhost"
#port = 5432
database = "journald_foo"
username = "journald_foo"
#password = ""

[data.journal]
frontend.name = "journald-foo"
frontend.data_type = "wide"
backend.name = "postgres-journald-foo"
backend.postgres_table = "journald_foo"
values.timestamp = "/__TIMESTAMP"
values.message = "/MESSAGE"
```

## Example journald data

Fields differ between units and possibly even messages.

```json
{
    "MESSAGE": "2025-03-05 22:20:02 0 [Note] /usr/bin/mariadbd: ready for connections.",
    "PRIORITY": "6",
    "SYSLOG_FACILITY": "3",
    "SYSLOG_IDENTIFIER": "mariadbd",
    "_BOOT_ID": "5cc2f07fb60cbff7d440e0998342c05b",
    "_CAP_EFFECTIVE": "100",
    "_COMM": "(mariadbd)",
    "_EXE": "/usr/bin/mariadbd",
    "_GID": "969",
    "_HOSTNAME": "logosII",
    "_MACHINE_ID": "e59ff97941044f85df5297e1c302d260",
    "_PID": "33940",
    "_RUNTIME_SCOPE": "system",
    "_STREAM_ID": "d8e8fca2dc0f896fd7cb4cb0031ba249",
    "_SYSTEMD_CGROUP": "/system.slice/mariadb.service",
    "_SYSTEMD_INVOCATION_ID": "d3b07384d113edec49eaa6238ad5ff00",
    "_SYSTEMD_SLICE": "system.slice",
    "_SYSTEMD_UNIT": "mariadb.service",
    "_TRANSPORT": "stdout",
    "_UID": "969"
}
```

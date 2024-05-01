# AhoyDTU

Frontend | Backend | Table Layout
--- | --- | ---
http-rest | postgres | wide

HTTP-REST setup for the AhoyDTU, an open-source module for controlling and reading data from
Hoymiles photovoltaik module power inverters.

## References

* <https://ahoydtu.de/>
* <https://ahoydtu.de/getting_started/>

## Setup of Postgres

```sql
-- Create User
CREATE USER pv;
-- Create Database
CREATE DATABASE pv OWNER pv;
REVOKE CONNECT ON DATABASE pv FROM PUBLIC;
-- connect to db
\c pv
-- Create Tables
SET ROLE pv;
CREATE TABLE IF NOT EXISTS measurements (
    timestamp timestamp with time zone NOT NULL,
    persistent bool NOT NULL,
    ac_voltage float4 NOT NULL,
    ac_current float4 NOT NULL,
    ac_power float4 NOT NULL,
    ac_frequency float4 NOT NULL,
    ac_power_factor float4 NOT NULL,
    ac_temperature float4 NOT NULL,
    ac_yield_total float4 NOT NULL,
    ac_yield_day float4 NOT NULL,
    ac_power_dc float4 NOT NULL,
    ac_efficiency float4 NOT NULL,
    ac_reactive_power float4 NOT NULL,
    ac_power_limit float4 NOT NULL,

    a_voltage float4 NOT NULL,
    a_current float4 NOT NULL,
    a_power float4 NOT NULL,
    a_yield_day float4 NOT NULL,
    a_yield_total float4 NOT NULL,
    a_irradiation float4 NOT NULL,

    b_voltage float4 NOT NULL,
    b_current float4 NOT NULL,
    b_power float4 NOT NULL,
    b_yield_day float4 NOT NULL,
    b_yield_total float4 NOT NULL,
    b_irradiation float4 NOT NULL,
    PRIMARY KEY (timestamp, persistent)
) PARTITION BY LIST(persistent);
CREATE TABLE measurements_persistent PARTITION OF measurements FOR VALUES IN (true);
CREATE TABLE measurements_nonpersistent PARTITION OF measurements FOR VALUES IN (false);
```

## Configuration of iot2db

```toml
[frontend.ahoydtu-rest]
type = "http-rest"
url = "http://IP.OF.THE.AHOYDTU/api/live"
#basic_auth = { username = "", password = "" }
frequency_secs = 10

[backend.postgres-pv]
type = "postgres"
host = "localhost"
#port = 5432
database = "pv"
username = "pv"
#password = ""

[data.pv]
frontend.name = "ahoydtu-rest"
backend.name = "postgres-pv"
backend.postgres_table = "measurements"
persistent_every_secs = 120
clean_non_persistent_after_days = 7
values.timestamp = { pointer = "/inverter/0/ts_last_success", postprocess = 'f"to_timestamp({value})"' }
values.ac_voltage = "/inverter/0/ch/0/0"
values.ac_current = "/inverter/0/ch/0/1"
values.ac_power = "/inverter/0/ch/0/2"
values.ac_frequency = "/inverter/0/ch/0/3"
values.ac_power_factor = "/inverter/0/ch/0/4"
values.ac_temperature = "/inverter/0/ch/0/5"
values.ac_yield_total = "/inverter/0/ch/0/6"
values.ac_yield_day = "/inverter/0/ch/0/7"
values.ac_power_dc = "/inverter/0/ch/0/8"
values.ac_efficiency = "/inverter/0/ch/0/9"
values.ac_reactive_power = "/inverter/0/ch/0/10"
values.ac_power_limit = "/inverter/0/power_limit_read"
values.a_voltage = "/inverter/0/ch/1/0"
values.a_current = "/inverter/0/ch/1/1"
values.a_power = "/inverter/0/ch/1/2"
values.a_yield_day = "/inverter/0/ch/1/3"
values.a_yield_total = "/inverter/0/ch/1/4"
values.a_irradiation = "/inverter/0/ch/1/5"
values.b_voltage = "/inverter/0/ch/2/0"
values.b_current = "/inverter/0/ch/2/1"
values.b_power = "/inverter/0/ch/2/2"
values.b_yield_day = "/inverter/0/ch/2/3"
values.b_yield_total = "/inverter/0/ch/2/4"
values.b_irradiation = "/inverter/0/ch/2/5"
```

## Example API Response

```json
{
  "menu": { "...": "..." },
  "generic": { "...": "..." },
  "inverter": [
    {
      "enabled": true,
      "name": "HM-800",
      "channels": 2,
      "power_limit_read": 100,
      "last_alarm": "Inverter start",
      "ts_last_success": 1691347360,
      "ch": [
        [
          232.2,
          0,
          0,
          50,
          0,
          16.6,
          248.13,
          1248,
          0.6,
          0,
          0
        ],
        [
          14.8,
          0.02,
          0.3,
          656,
          132.329,
          0.071
        ],
        [
          14.8,
          0.02,
          0.3,
          592,
          115.801,
          0.071
        ]
      ],
      "ch_names": [
        "AC",
        "A",
        "B"
      ]
    }
  ],
  "refresh_interval": 5,
  "ch0_fld_units": [
    "V",
    "A",
    "W",
    "Hz",
    "",
    "°C",
    "kWh",
    "Wh",
    "W",
    "%",
    "var"
  ],
  "ch0_fld_names": [
    "U_AC",
    "I_AC",
    "P_AC",
    "F_AC",
    "PF_AC",
    "Temp",
    "YieldTotal",
    "YieldDay",
    "P_DC",
    "Efficiency",
    "Q_AC"
  ],
  "fld_units": [
    "V",
    "A",
    "W",
    "Wh",
    "kWh",
    "%"
  ],
  "fld_names": [
    "U_DC",
    "I_DC",
    "P_DC",
    "YieldDay",
    "YieldTotal",
    "Irradiation"
  ]
}
```

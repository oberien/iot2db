# EVCC

Frontend | Backend | Table Layout
--- | --- | ---
mqtt (narrow) | postgres | wide

[evcc](https://evcc.io/) is an open-source PV house battery and car charging manager.
It supports value output via mqtt.

## References

* [evcc documentation of mqtt](https://docs.evcc.io/docs/reference/configuration/mqtt)

## Configuration of evcc

Add the following section to your `/etc/evcc.yaml`:
```yaml
mqtt:
  broker: 192.168.x.y
  topic: evcc
```

Restart evcc (`systemctl restart evcc`).
 
## Setup of Postgres

```sql
-- Create User
CREATE USER evcc;
-- Create Database
CREATE DATABASE evcc OWNER evcc;
REVOKE CONNECT ON DATABASE evcc FROM PUBLIC;
-- connect to db
\c evcc
-- Create Tables
SET ROLE evcc;
CREATE TABLE IF NOT EXISTS measurements (
    timestamp timestamp with time zone NOT NULL,
    persistent bool NOT NULL DEFAULT false,
    car_soc float8,
    car_connected bool,
    car_charging bool,
    car_power float8,
    battery_soc float8,
    pv_power float8,
    battery_power float8,
    grid_power float8,
    home_power float8,
    total_charged_kwh float8,
    total_solar_percentage float8,
    PRIMARY KEY (timestamp, persistent)
) PARTITION BY LIST(persistent);
CREATE TABLE measurements_persistent PARTITION OF measurements FOR VALUES IN (true);
CREATE TABLE measurements_nonpersistent PARTITION OF measurements FOR VALUES IN (false);
```

## Configuration of iot2db

```toml
[frontend.mqtt]
type = "mqtt"
host = "IP.OF.MQTT.BROKER"
#port = 1883
#username = ""
#password = ""

[backend.postgres-evcc]
type = "postgres"
host = "localhost"
#port = 5432
database = "evcc"
username = "evcc"
#password = ""

[data.evcc]
frontend.name = "mqtt"
frontend.mqtt_topic = "evcc/#"
frontend.data_type = "narrow"
backend.name = "postgres-evcc"
backend.postgres_table = "measurements"
persistent_every_secs = 120
clean_non_persistent_after_days = 7
values.timestamp = { constant_value = "", postprocess = '"CURRENT_TIMESTAMP"' }
values.car_soc = "/evcc~1loadpoints~11~1vehicleSoc"
values.car_connected = "/evcc~1loadpoints~11~1connected"
values.car_charging = "/evcc~1loadpoints~11~1charging"
values.car_power = "/evcc~1loadpoints~11~1chargePower"
values.battery_soc = "/evcc~1site~1batterySoc"
values.pv_power = "/evcc~1site~1pvPower"
values.battery_power = "/evcc~1site~1batteryPower"
values.grid_power = "/evcc~1site~1gridPower"
values.home_power = "/evcc~1site~1homePower"
values.total_charged_kwh = "/evcc~1site~1statistics~1total~1chargedKWh"
values.total_solar_percentage = "/evcc~1site~1statistics~1total~1solarPercentage"
```

## Example MQTT Messages

```
evcc/loadpoints/1/vehicleSoc 67
evcc/loadpoints/1/connected true
evcc/loadpoints/1/charging false
evcc/loadpoints/1/chargePower 0.3
evcc/site/batterySoc 25
evcc/site/pvPower 0
evcc/site/batteryPower 206
evcc/site/gridPower -7.5
evcc/site/homePower 198.2
evcc/site/statistics/total/chargedKWh 44.679
evcc/site/statistics/total/solarPercentage 63.918
```

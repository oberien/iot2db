# AirQ Science

Frontend | Backend | Table Layout
--- | --- | ---
mqtt | postgres | wide

AirQ is an air quality measurement device with lots of different sensors.
The AirQ Science (and only science) supports MQTT.

## References

* <https://support.air-q.com/en/support/does-the-air-q-support-mqtt/>
* <https://docs.air-q.com/> > Data transmission and management > MQTT

## Configuration of the AirQ

* power off the AirQ
* pull out the microSD-Card
* connect and mount the microSD card on the PC
* add `config.json` to the microSD, adapting it to your needs
* copied from the air-q docs:
    > 
    > ```json
    > {
    >   "mqtt": {
    >     "device_id": "Your_self-defined_device_ID",
    >     "broker_URL": "192.168.x.y",
    >     "user": "Your_Username",
    >     "password": "Your_Password",
    >     "port": 8883,
    >     "topic": "Your_Topic",
    >     "retain": false,
    >     "qos": 1,
    >     "keepalive": 10000,
    >     "averages": true,
    >     "delay": 120,
    >     "ssl": true,
    >     "ssl_params": {
    >       "cert_reqs": 2,
    >       "certfile": "/sd/cert/certfile",
    >       "keyfile": "/sd/cert/keyfile",
    >       "ca_certs": "/sd/cert/cafile"
    >     }
    >   }
    > }
    > ```
    > The values `device_id`, `user`, `password`, `qos`, `retain`, `keepalive`, `averages`, and `delay` can be omitted.
    > Then standard values will be assumed.
    > If device_id isn’t given, the actual air-Q devices ID will be added to the MQTT message.
    > 
    > If `ssl` is `true`, the certificate files have to be on the sd card in a directory named `cert/`
    > as given by your setting of `ssl_params`.
 
## Setup of Postgres

```sql
-- Create User
CREATE USER airq;
-- Create Database
CREATE DATABASE airq OWNER airq;
REVOKE CONNECT ON DATABASE airq FROM PUBLIC;
-- connect to db
\c airq
-- Create Tables
SET ROLE airq;
CREATE TABLE IF NOT EXISTS measurements (
    timestamp timestamp with time zone NOT NULL,
    persistent bool NOT NULL DEFAULT false,
    health float8 NOT NULL,
    performance float8 NOT NULL,
    tvoc float8,
    humidity float8 NOT NULL,
    humidity_abs float8 NOT NULL,
    temperature float8 NOT NULL,
    dewpt float8 NOT NULL,
    sound float8 NOT NULL,
    pressure float8 NOT NULL,
    no2 float8,
    co float8,
    co2 float8 NOT NULL,
    pm1 float8 NOT NULL,
    pm2_5 float8 NOT NULL,
    pm10 float8 NOT NULL,
    oxygen float8 NOT NULL,
    o3 float8,
    so2 float8,
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

[backend.postgres-airq]
type = "postgres"
host = "localhost"
#port = 5432
database = "airq"
username = "airq"
#password = ""

[data.airq]
frontend.name = "mqtt"
frontend.mqtt_topic = "Your_Topic"
backend.name = "postgres-airq"
backend.postgres_table = "measurements"
persistent_every_secs = 120
clean_non_persistent_after_days = 7
values.timestamp = { pointer = "/timestamp", preprocess = 'f"{value.parse_int().unwrap()/1000}"', postprocess = 'f"to_timestamp({value})"' }
values.health = "/health"
values.performance = "/performance"
values.tvoc = "/tvoc/0"
values.humidity = "/humidity/0"
values.humidity_abs = "/humidity_abs/0"
values.temperature = "/temperature/0"
values.dewpt = "/dewpt/0"
values.sound = "/sound/0"
values.pressure = "/pressure/0"
values.no2 = "/no2/0"
values.co = "/co/0"
values.co2 = "/co2/0"
values.pm1 = "/pm1/0"
values.pm2_5 = "/pm2_5/0"
values.pm10 = "/pm10/0"
values.oxygen = "/oxygen/0"
values.o3 = "/o3/0"
values.so2 = "/so2/0"
```

## Example MQTT Message

```json
Your_Topic {
  "oxygen": [20.442, 4.33],
  "health": 992,
  "temperature": [25.098, 0.53],
  "dewpt": [16.823, 0.99],
  "timestamp": 1690715588000,
  "Status": {
    "co": "co sensor still in warm up phase; waiting time = 582 s",
    "so2": "so2 sensor still in warm up phase; waiting time = 2300 s",
    "no2": "no2 sensor still in warm up phase; waiting time = 2300 s",
    "o3": "o3 sensor still in warm up phase; waiting time = 2300 s"
  },
  "sound": [57.05, 2.9],
  "humidity": [59.398, 4.49],
  "tvoc": [157, 24],
  "sound_max": [83.6, 1.9],
  "pm10": [0.8, 10.0],
  "pressure": [959.0, 1.0],
  "co2": [440.7, 63.2],
  "DeviceID": "Your_self-defined_device_ID",
  "performance": 874,
  "pm2_5": [0.3, 10.0],
  "TypPS": 4.6,
  "pm1": [0.1, 10.0],
  "humidity_abs": [13.81, 0.86]
}
```

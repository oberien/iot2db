# Homematic CCU3 / RaspberryMatic

Frontend | Backend | Table Layout
--- | --- | ---
homematic-ccu3 | postgres | wide

HTTP setup for a self-hosted RaspberryMatic / Homematic CCU3.

## References

* 

## Setup

* create a new user "iot2db" with permissions "User"

## Example Response

```json
[{
  "address": "000A1234567890",
  "channels": [
    {
      "address": "000A1234567890:0",
      "category": "CATEGORY_NONE",
      "channelType": "MAINTENANCE",
      "deviceId": "1337",
      "id": "1338",
      "index": 0,
      "isAesAvailable": false,
      "isEventable": true,
      "isInternal": false,
      "isLogable": true,
      "isLogged": false,
      "isReadable": true,
      "isReady": true,
      "isUsable": true,
      "isVirtual": false,
      "isVisible": true,
      "isWritable": false,
      "master": {
        "ARR_TIMEOUT": "10",
        "CYCLIC_INFO_MSG": "1",
        "CYCLIC_INFO_MSG_DIS": "1",
        "CYCLIC_INFO_MSG_DIS_UNCHANGED": "20",
        "CYCLIC_INFO_MSG_OVERDUE_THRESHOLD": "2",
        "DAYLIGHT_SAVINGS_TIME": "1",
        "DST_END_DAY_OF_WEEK": "0",
        "DST_END_MONTH": "10",
        "DST_END_TIME": "180",
        "DST_END_WEEK_OF_MONTH": "5",
        "DST_START_DAY_OF_WEEK": "0",
        "DST_START_MONTH": "3",
        "DST_START_TIME": "120",
        "DST_START_WEEK_OF_MONTH": "5",
        "DUTYCYCLE_LIMIT": "180",
        "ENABLE_ROUTING": "1",
        "GLOBAL_BUTTON_LOCK": "0",
        "LOCAL_RESET_DISABLED": "0",
        "LOW_BAT_LIMIT": "2.200000",
        "UTC_DST_OFFSET": "120",
        "UTC_OFFSET": "60"
      },
      "mode": "MODE_AES",
      "name": "Thermostat 1:0",
      "partnerId": "",
      "values": {
        "CONFIG_PENDING": "0",
        "DUTY_CYCLE": "0",
        "LOW_BAT": "0",
        "OPERATING_VOLTAGE": "2.500000",
        "OPERATING_VOLTAGE_STATUS": "0",
        "RSSI_DEVICE": "-83",
        "UNREACH": "0",
        "UPDATE_PENDING": "0"
      }
    },
    {
      "address": "000A1234567890:1",
      "category": "CATEGORY_SENDER",
      "channelType": "HEATING_CLIMATECONTROL_TRANSCEIVER",
      "deviceId": "1337",
      "id": "1339",
      "index": 1,
      "isAesAvailable": false,
      "isEventable": true,
      "isInternal": false,
      "isLogable": true,
      "isLogged": false,
      "isReadable": true,
      "isReady": true,
      "isUsable": true,
      "isVirtual": false,
      "isVisible": true,
      "isWritable": true,
      "master": {
        "ADAPTIVE_REGULATION": "2",
        "BOOST_AFTER_WINDOW_OPEN": "0",
        "BOOST_POSITION": "80",
        "BOOST_TIME_PERIOD": "5",
        "BUTTON_RESPONSE_WITHOUT_BACKLIGHT": "0",
        "CHANNEL_OPERATION_MODE": "0",
        "DECALCIFICATION_TIME": "22",
        "DECALCIFICATION_WEEKDAY": "6",
        "DURATION_5MIN": "0",
        "MANU_MODE_PRIORITIZATION": "1",
        "MIN_MAX_VALUE_NOT_RELEVANT_FOR_MANU_MODE": "0",
        "OPTIMUM_START_STOP": "0",
        "P1_ENDTIME_FRIDAY_1": "360",
        "...": "...",
        "P3_TEMPERATURE_WEDNESDAY_9": "17.000000",
        "PARTY_MODE_PRIORITIZATION": "1",
        "TEMPERATUREFALL_MODUS": "4",
        "TEMPERATUREFALL_VALUE": "1.400000",
        "TEMPERATUREFALL_WINDOW_OPEN_TIME_PERIOD": "15",
        "TEMPERATURE_COMFORT": "21.000000",
        "TEMPERATURE_LOWERING": "17.000000",
        "TEMPERATURE_MAXIMUM": "30.500000",
        "TEMPERATURE_MINIMUM": "4.500000",
        "TEMPERATURE_OFFSET": "0.000000",
        "TEMPERATURE_WINDOW_OPEN": "5.000000",
        "VALVE_ERROR_RUN_POSITION": "0.150000",
        "VALVE_MAXIMUM_POSITION": "1.000000",
        "VALVE_OFFSET": "0.000000"
      },
      "mode": "MODE_AES",
      "name": "Thermostat 1:1",
      "partnerId": "",
      "values": {
        "ACTIVE_PROFILE": "1",
        "ACTUAL_TEMPERATURE": "23.300000",
        "ACTUAL_TEMPERATURE_STATUS": "0",
        "BOOST_MODE": "0",
        "BOOST_TIME": "0",
        "FROST_PROTECTION": "0",
        "LEVEL": "0.000000",
        "LEVEL_STATUS": "0",
        "PARTY_MODE": "0",
        "QUICK_VETO_TIME": "0",
        "SET_POINT_MODE": "1",
        "SET_POINT_TEMPERATURE": "5.000000",
        "SWITCH_POINT_OCCURED": "0",
        "VALVE_STATE": "4",
        "WINDOW_STATE": "1"
      }
    },
    "..."
  ],
  "enabledServiceMsg": "true",
  "id": "1337",
  "interface": "HmIP-RF",
  "isReady": "true",
  "name": "Thermostat 1",
  "operateGroupOnly": "false",
  "type": "HmIP-eTRV-2"
},{
  "address": "00AB1234567890",
  "channels": [
    {
      "address": "00AB1234567890:0",
      "category": "CATEGORY_NONE",
      "channelType": "MAINTENANCE",
      "deviceId": "2337",
      "id": "2338",
      "index": 0,
      "isAesAvailable": false,
      "isEventable": true,
      "isInternal": false,
      "isLogable": true,
      "isLogged": false,
      "isReadable": true,
      "isReady": true,
      "isUsable": true,
      "isVirtual": false,
      "isVisible": true,
      "isWritable": false,
      "master": {
        "ARR_TIMEOUT": "10",
        "CYCLIC_BIDI_INFO_MSG_DISCARD_FACTOR": "1",
        "CYCLIC_BIDI_INFO_MSG_DISCARD_VALUE": "57",
        "CYCLIC_INFO_MSG": "1",
        "CYCLIC_INFO_MSG_DIS": "1",
        "CYCLIC_INFO_MSG_DIS_UNCHANGED": "20",
        "CYCLIC_INFO_MSG_OVERDUE_THRESHOLD": "2",
        "DISABLE_MSG_TO_AC": "0",
        "DUTYCYCLE_LIMIT": "180",
        "ENABLE_ROUTING": "1",
        "LOCAL_RESET_DISABLED": "0",
        "LOW_BAT_LIMIT": "1.100000",
        "SUPPORTING_WIRED_OPERATION_MODE": "1"
      },
      "mode": "MODE_AES",
      "name": "Window 1:0",
      "partnerId": "",
      "values": {
        "CONFIG_PENDING": "0",
        "DUTY_CYCLE": "0",
        "ERROR_CODE": "0",
        "LOW_BAT": "0",
        "OPERATING_VOLTAGE": "1.400000",
        "OPERATING_VOLTAGE_STATUS": "0",
        "RSSI_DEVICE": "-64",
        "SABOTAGE": "0",
        "UNREACH": "0",
        "UPDATE_PENDING": "0"
      }
    },
    {
      "address": "00AB1234567890:1",
      "category": "CATEGORY_SENDER",
      "channelType": "SHUTTER_CONTACT",
      "deviceId": "2337",
      "id": "2339",
      "index": 1,
      "isAesAvailable": false,
      "isEventable": true,
      "isInternal": false,
      "isLogable": true,
      "isLogged": false,
      "isReadable": true,
      "isReady": true,
      "isUsable": true,
      "isVirtual": false,
      "isVisible": true,
      "isWritable": false,
      "master": {
        "ALARM_MODE_TYPE": "0",
        "ALARM_MODE_ZONE_1": "0",
        "ALARM_MODE_ZONE_2": "0",
        "ALARM_MODE_ZONE_3": "0",
        "ALARM_MODE_ZONE_4": "0",
        "ALARM_MODE_ZONE_5": "0",
        "ALARM_MODE_ZONE_6": "0",
        "ALARM_MODE_ZONE_7": "0",
        "EVENT_DELAY_UNIT": "0",
        "EVENT_DELAY_VALUE": "10",
        "MSG_FOR_POS_A": "2",
        "MSG_FOR_POS_B": "1",
        "SAMPLE_INTERVAL": "5.000000"
      },
      "mode": "MODE_AES",
      "name": "Window 1:1",
      "partnerId": "",
      "values": {
        "STATE": "1"
      }
    },
    "..."
  ],
  "enabledServiceMsg": "true",
  "id": "2337",
  "interface": "HmIP-RF",
  "isReady": "true",
  "name": "Window 1",
  "operateGroupOnly": "false",
  "type": "HmIP-SWDO-2"
}]
```

## Setup Postgres

```sql
-- Create User
CREATE USER homematic;
-- Create Database
CREATE DATABASE homematic OWNER homematic;
REVOKE CONNECT ON DATABASE homematic FROM PUBLIC;
-- connect to db
\c homematic
-- Create Tables
SET ROLE homematic;
CREATE TABLE IF NOT EXISTS measurements (
    timestamp timestamp with time zone NOT NULL,
    persistent bool NOT NULL,
    thermostat_voltage float4 NOT NULL,
    thermostat_rssi int2 NOT NULL,
    thermostat_temp float4 NOT NULL,
    thermostat_desired_temp float4 NOT NULL,
    thermostat_valve float4 NOT NULL,
    thermostat_window_open bool NOT NULL,
    window_voltage float4 NOT NULL,
    window_rssi int2 NOT NULL,
    window_open bool NOT NULL,
    PRIMARY KEY (timestamp, persistent)
) PARTITION BY LIST(persistent);
CREATE TABLE measurements_persistent PARTITION OF measurements FOR VALUES IN (true);
CREATE TABLE measurements_nonpersistent PARTITION OF measurements FOR VALUES IN (false);
```

## Configuration of iot2db

```toml
[frontend.homematic]
type = "homematic-ccu3"
url = "https://ccu3-url"
#basic_auth = { username = "", password = "" }
frequency_secs = 10
username = "iot2db"
password = "MyPassword"

[backend.postgres-homematic]
type = "postgres"
host = "localhost"
#port = 5432
database = "homematic"
username = "homematic"
#password = ""

[data.homematic]
frontend.name = "homematic"
backend.name = "postgres-pv"
backend.postgres_table = "measurements"
persistent_every_secs = 120
clean_non_persistent_after_days = 7
values.timestamp = { pointer = "/nonexistent", postprocess = '"CURRENT_TIMESTAMP"' }
values.thermostat_voltage = "/Thermostat 1/channels/0/values/OPERATING_VOLTAGE"
values.thermostat_rssi = "/Thermostat 1/channels/0/values/RSSI_DEVICE"
values.thermostat_temp = "/Thermostat 1/channels/1/values/ACTUAL_TEMPERATURE"
values.thermostat_desired_temp = "/Thermostat 1/channels/1/values/SET_POINT_TEMPERATURE"
values.thermostat_valve = "/Thermostat 1/channels/1/values/LEVEL"
values.thermostat_window_open = "/Thermostat 1/channels/1/values/WINDOW_STATE"
values.window_voltage = "/Window 1/channels/0/values/OPERATING_VOLTAGE"
values.window_rssi = "/Window 1/channels/0/values/RSSI_DEVICE"
values.window_open = "/Window 1/channels/1/values/STATE"
```

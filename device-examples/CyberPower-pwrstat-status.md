# CyberPower UPS `pwrstat -status`

Frontend | Backend | Table Layout
--- | --- | ---
shell (wide via regex) | postgres | wide

Gather data from a CyberPower UPS using CyberPower's `pwrtstat -status` cli.

## References

* 

## Setup of Postgres

```sql
-- Create User
CREATE USER ups;
-- Create Database
CREATE DATABASE ups OWNER ups;
REVOKE CONNECT ON DATABASE ups FROM PUBLIC;
-- connect to db
\c ups
-- Create Tables
SET ROLE ups;
CREATE TABLE IF NOT EXISTS measurements (
    timestamp timestamp with time zone NOT NULL,
    persistent bool NOT NULL,
    state text NOT NULL,
    power_supply_by text NOT NULL,
    utility_voltage int2 NOT NULL,
    output_voltage int2 NOT NULL,
    battery_capacity int2 NOT NULL,
    remaining_runtime int2 NOT NULL,
    load int2 NOT NULL,
    line_interaction text NOT NULL,
    test_result text NULL,
    test_result_date timestamp with time zone NULL,
    last_power_event text NULL,
    last_power_event_date timestamp with time zone NULL,
    last_power_event_duration interval NULL,
    PRIMARY KEY (timestamp, persistent)
) PARTITION BY LIST(persistent);
CREATE TABLE measurements_persistent PARTITION OF measurements FOR VALUES IN (true);
CREATE TABLE measurements_nonpersistent PARTITION OF measurements FOR VALUES IN (false);
```

## Configuration of iot2db

```toml
[frontend.pwrstat]
type = "shell"
cmd = "pwrstat -status"
frequency_secs = 10
regex.state = "State\\.+ ([^\\n]+)"
regex.power_supply_by = "Power Supply by\\.+ ([^\\n]+)"
regex.utility_voltage = "Utility Voltage\\.+ (\\d+)"
regex.output_voltage = "Output Voltage\\.+ (\\d+)"
regex.battery_capacity = "Battery Capacity\\.+ (\\d+)"
regex.remaining_runtime = "Remaining Runtime\\.+ (\\d+)"
regex.load = "Load\\.+ (\\d+)"
regex.line_interaction = "Line Interaction\\.+ ([^\\n]+)"
regex.test_result = "Test Result\\.+ (.*) at "
regex.test_result_date = "Test Result\\.+ .+ at (\\d{4}/\\d{2}/\\d{2} \\d{2}:\\d{2}:\\d{2})"
regex.last_power_event = "Last Power Event\\.+ (.+) at "
regex.last_power_event_date = "Last Power Event\\.+ .+ at (\\d{4}/\\d{2}/\\d{2} \\d{2}:\\d{2}:\\d{2})"
regex.last_power_event_duration = "Last Power Event\\.+ .+ for (.*)\\.\\n"

[backend.pwrstat]
type = "postgres"
host = "localhost"
database = "ups"
username = "ups"

[data.pwrstat]
frontend.name = "pwrstat"
frontend.data_type = "wide"
backend.name = "pwrstat"
backend.postgres_table = "measurements"
persistent_every_secs = 120
clean_non_persistent_after_days = 7
direct_keys = "all"
values.timestamp = { constant_value = "", postprocess = '"CURRENT_TIMESTAMP"' }
```

## Example `pwrstat -status` output

```text

The UPS information shows as following:

        Properties:
                Model Name................... Value2200E
                Firmware Number.............. BZCB102#41A
                Rating Voltage............... 230 V
                Rating Power................. 1320 Watt(2200 VA)

        Current UPS status:
                State........................ Normal
                Power Supply by.............. Utility Power
                Utility Voltage.............. 230 V
                Output Voltage............... 232 V
                Battery Capacity............. 97 %
                Remaining Runtime............ 24 min.
                Load......................... 290 Watt(22 %)
                Line Interaction............. None
                Test Result.................. Passed at 2024/04/19 16:26:52
                Last Power Event............. Blackout at 2024/05/01 17:35:30
```

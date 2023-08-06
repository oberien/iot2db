# iot2db

A service to fetch data from MQTT or a REST API and store it in a database
(e.g. for later visualization with Grafana).

Currently supported:
* Frontends
    * HTTP REST
    * MQTT
* Backends:
    * PostgreSQL

## Installation

* requires an installation of rust (see <https://rustup.rs>)
    ```sh
    make
    sudo make install
    ```
* edit the config file `/etc/iot2db.toml`
    * see `device-examples/` for setup and configuration examples of different devices
* `systemctl enable iot2db`
* `systemctl start iot2db`
* uninstall using `sudo make uninstall`
    * keeps the config file `/etc/iot2db.toml` - delete manually if wanted

## Table Layouts

Wide (`wide`):
* every device has its own table for measurements
* every measurement is one column
* e.g. `timestamp (time), co2 (float8), voc (int4), humidity (float4)`
* Advantages:
    * each measurement has its correct type
    * queries must be adapted for different devices
    * hard to e.g. get the temperature of multiple different devices
* Disadvantages:
    * each different device requires its own new table
    * only useful if each scan report includes all data

Narrow (`narrow`):
* currently _not_ supported
* one table for all devices and measurements
* e.g. `timestamp (time), device (text), measurement (text), value (float8)`
* Advantages:
    * one table for all devices and measurements
    * adding a new device or measurement doesn't affect database schema
* Disadvantages:
    * only one type for all measurements
    * large (~10x) per-value overhead (device and measurement strings stored lots of times)

Narrow M:N (`narrow-mn`):
* currently _not_ supported
* one table for all devices and measurements
* one table for measurement-names (`id, measurement (text)`)
* one table for device-names (`id, measurement (text)`)
* e.g. `timestamp (time), device (id-ref), measurement (id-ref), value (float8)`
* Advantages:
    * same as `narrow`
    * additionally less per-value overhead (only 2 IDs)
* Disadvantages:
    * only one type for all measurements
    * inserts become more complicated as they may need to insert into the measurement- and device-names
    * still a bit more overhead compared to `wide`

Medium (`medium`) / Medium M:N (`medium-mn`)
* currently _not_ supported
* similar to `narrow` / `narrow-mn`
* solves the one-type-for-all-measurements problem by having one column per possible type
* e.g. `timestamp (time), device, measurement, float_value (float8), int_value (int8)`
* Advantages:
    * each value can have its specific type
* Disadvantages:
    * makes querying harder as one needs to remember each measurement's respective type-column

References:
* table layouts: <https://www.timescale.com/blog/best-practices-for-time-series-data-modeling-narrow-medium-or-wide-table-layout-2/>
* table layout overheads: <https://dba.stackexchange.com/a/231292>
* optimizing wide tables: <https://aws.amazon.com/de/blogs/database/designing-high-performance-time-series-data-tables-on-amazon-rds-for-postgresql/>

# License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

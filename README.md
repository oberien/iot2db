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
* uninstall using `sudo make uninstall`
    * keeps the config file `/etc/iot2db.toml` - delete manually if wanted

## Configuration

* config File: `/etc/iot2db.toml`
* see `device-examples/` for setup and configuration examples of different devices

# License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](/LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

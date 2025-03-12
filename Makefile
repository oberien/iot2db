.PHONY: all
all: build

.PHONY: build
build:
	cargo build --release

.PHONY: install
install:
	install -D -m 755 -o root -g root target/release/iot2db /usr/local/bin/iot2db
	install -D -m 640 -o root -g root iot2db.service /usr/lib/systemd/system/
	systemctl daemon-reload
	if [ ! -f /etc/iot2db.toml ]; then install -D -m 644 -o root -g root iot2db-example.toml /etc/iot2db.toml; fi

.PHONY: uninstall
uninstall:
	systemctl stop iot2db.service
	systemctl disable iot2db.service
	rm /usr/local/bin/iot2db
	rm /usr/lib/systemd/system/iot2db.service

.PHONY: clean
clean:
	cargo clean

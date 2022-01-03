#!/bin/sh
cargo build
sudo systemctl stop batt-monitor
sudo cp target/debug/batt_monitor /usr/bin/
sudo systemctl start batt-monitor
journalctl -u batt-monitor -f


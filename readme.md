## gpt 4.1 generated readme ##

# wgtoggle

**wgtoggle** is a lightweight Rust daemon that automatically toggles your WireGuard VPN connection based on your current WiFi SSID. It listens for network connection changes via NetworkManager and starts or stops the WireGuard VPN accordingly.

---

## Features

- Reacts to NetworkManager WiFi state changes
- Automatically connects WireGuard VPN when off home WiFi and disconnects WireGuard VPN when on specified home WiFi

---

## Requirements

- Linux system with NetworkManager
- WireGuard configured as a NetworkManager connection
- Rust toolchain to build (optional if using prebuilt binary)

---

## Configuration

**Important:** You must set the following environment variables; there are no default values.

| Variable             | Description                             |
|----------------------|---------------------------------------|
| `HOME_SSID`          | Your home WiFi SSID name               |
| `WIREGUARD_VPN_NAME` | NetworkManager WireGuard connection name |

If either is missing, the daemon logs an error and stops execution.

---

## Building

```bash
git clone <repository-url>
cd wgtoggle
cargo build --release
sudo cp target/release/wgtoggle /usr/local/bin/
sudo chmod +x /usr/local/bin/wgtoggle

---

## Usage

sudo HOME_SSID="MyHomeWiFi" WIREGUARD_VPN_NAME="wg0" /usr/local/bin/wgtoggle

** OR create systemd service **

** to check your wireguard connection name type nmcli connection show and look for type wireguard **

sudo nano /etc/systemd/system/wgtoggle.service

with

[Unit]
Description=WireGuard Auto Toggle Daemon
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/wgtoggle
Restart=on-failure
RestartSec=5
User=root
Environment=RUST_LOG=info

# You MUST set these for the service to run properly 
Environment=HOME_SSID=my-home-wifi
Environment=WIREGUARD_VPN_NAME=my-wireguard-vpn

[Install]
WantedBy=multi-user.target

then

sudo systemctl daemon-reload
sudo systemctl enable --now wgtoggle.service
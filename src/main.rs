use futures_util::stream::StreamExt;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::env;
use tokio::process::Command;
use zbus::{Connection, Result, proxy};

#[proxy(
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager",
    interface = "org.freedesktop.NetworkManager"
)]
trait NetworkManager {
    #[zbus(signal)]
    fn state_changed(&self, state: u32) -> zbus::Result<()>;
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::new()
        .env()
        .init()
        .expect("logger failed to initialize");
    #[allow(unused_assignments)]
    let mut home_ssid = String::new();
    match env::var("HOME_SSID") {
        Ok(s) => {
            home_ssid = s;
            info!("Configured home SSID: {}", home_ssid);
        }
        Err(_) => {
            error!("no configured HOME_SSID name in enviroment");
            panic!()
        }
    };
    #[allow(unused_assignments)]
    let mut wireguard_vpn_name = String::new();
    match env::var("WIREGUARD_VPN_NAME") {
        Ok(s) => {
            wireguard_vpn_name = s;
            info!("Configured WireGuard VPN name: {}", wireguard_vpn_name);
        }
        Err(_) => {
            error!("no configured WIREGUARD_VPN_NAME name in enviroment");
            panic!()
        }
    };

    let connection = Connection::system().await?;
    let proxy = NetworkManagerProxy::new(&connection).await?;
    let mut signal_stream = proxy.receive_state_changed().await?;
    info!("📡 Listening for NetworkManager 'StateChanged' signals...");
    while let Some(signal) = signal_stream.next().await {
        let args: StateChangedArgs = match signal.args() {
            Ok(a) => a,
            Err(e) => {
                error!("Error parsing StateChanged signal arguments: {}", e);
                continue;
            }
        };
        let state = args.state;
        match state {
            60 => wireguard_toggle(&wireguard_vpn_name, &home_ssid).await,
            _ => (),
        };
    }

    error!("❌ Signal stream ended unexpectedly");
    panic!();
}

async fn wireguard_toggle(vpn_name: &str, ssid: &str) {
    info!("Network change detected, checking SSID...");

    // Get current active SSID
    let output = match Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
        .await
    {
        Ok(out) => out,
        Err(e) => {
            error!("❌ Failed to run nmcli command: {}", e);
            return;
        }
    };

    if !output.status.success() {
        error!("❌ nmcli did not execute successfully: {}", output.status);
        return;
    }

    // Find active SSID from nmcli output
    let mut active_ssid = String::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.starts_with("yes:") {
            active_ssid = line.replacen("yes:", "", 1);
            break;
        }
    }

    let vpn_name = vpn_name;

    if active_ssid == ssid {
        info!(
            "Connected to home wifi '{}', stopping WireGuard VPN...",
            active_ssid
        );
        stop_vpn(vpn_name).await
    } else {
        info!(
            "Connected to SSID '{}', starting WireGuard VPN...",
            active_ssid
        );
        start_vpn(vpn_name).await
    }
}

async fn start_vpn(vpn_name: &str) {
    match Command::new("nmcli")
        .args(&["connection", "up", "id", vpn_name])
        .output()
        .await
    {
        Ok(status) => {
            if status.status.success() {
                info!("nmcli started WireGuard VPN '{}'", vpn_name);
            } else {
                let stderr = String::from_utf8_lossy(&status.stderr);
                error!("nmcli failed to start VPN '{}': {}", vpn_name, stderr);
            }
        }
        Err(e) => {
            error!("Failed to run nmcli command: {}", e);
        }
    }
}

async fn stop_vpn(vpn_name: &str) {
    match Command::new("nmcli")
        .args(&["connection", "down", "id", vpn_name])
        .output()
        .await
    {
        Ok(status) => {
            if status.status.success() {
                info!("nmcli stopped WireGuard VPN '{}'", vpn_name);
            } else {
                let stderr = String::from_utf8_lossy(&status.stderr);
                error!("nmcli failed to stop VPN '{}': {}", vpn_name, stderr);
            }
        }
        Err(e) => {
            error!("Failed to run nmcli command: {}", e);
        }
    }
}

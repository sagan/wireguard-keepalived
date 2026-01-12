use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Configuration constants
const HANDSHAKE_TIMEOUT_SEC: u64 = 180;
const CHECK_INTERVAL_SEC: u64 = 25;

fn main() {
    println!("Starting wireguard-keepalived...");
    println!(
        "Monitor: Handshakes > {}s on peers with Keepalive set.",
        HANDSHAKE_TIMEOUT_SEC
    );
    println!("Action: 'wg set <interface> listen-port 0'");

    loop {
        if let Err(e) = check_and_recover() {
            eprintln!("Error during check cycle: {}", e);
        }
        thread::sleep(Duration::from_secs(CHECK_INTERVAL_SEC));
    }
}

fn check_and_recover() -> std::io::Result<()> {
    // 1. Run "wg show all dump"
    // Format: intf, peer_pub, psk, endpoint, allowed_ips, latest_handshake, rx, tx, persistent_keepalive
    let output = Command::new("wg")
        .arg("show")
        .arg("all")
        .arg("dump")
        .output()?;

    if !output.status.success() {
        eprintln!("'wg show all dump' failed");
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Use a Set to avoid resetting the same interface multiple times in one cycle
    let mut stale_interfaces = HashSet::new();

    for line in stdout.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();

        // Basic validation of the dump line format (needs at least 9 fields)
        if fields.len() < 9 {
            continue;
        }

        let interface = fields[0];
        let latest_handshake_str = fields[5];
        let keepalive_str = fields[8];

        // 2. Filter: Must have PersistentKeepalive set (not "off" and not "0")
        let keepalive = keepalive_str.parse::<u64>().unwrap_or(0);
        if keepalive == 0 {
            continue;
        }

        // 3. Check Handshake Age
        let latest_handshake = latest_handshake_str.parse::<u64>().unwrap_or(now);
        let age = now.saturating_sub(latest_handshake);

        if age > HANDSHAKE_TIMEOUT_SEC {
            if !stale_interfaces.contains(interface) {
                println!(
                    "[{}] Stale detected! Interface: {}, Peer Keepalive: {}, Handshake Age: {}s",
                    now, interface, keepalive, age
                );
                stale_interfaces.insert(interface.to_string());
            }
        }
    }

    // 4. Action: Reset port for stale interfaces
    for interface in stale_interfaces {
        randomize_listen_port(&interface)?;
    }

    Ok(())
}

fn randomize_listen_port(interface: &str) -> std::io::Result<()> {
    println!(" -> Randomizing listen-port for '{}'...", interface);

    let status = Command::new("wg")
        .arg("set")
        .arg(interface)
        .arg("listen-port")
        .arg("0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        println!(" -> Success.");
    } else {
        eprintln!(" -> Failed to set listen-port.");
    }

    Ok(())
}

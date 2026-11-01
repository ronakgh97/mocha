pub mod monitor;
pub mod ui;
use anyhow::Result;

fn main() -> Result<()> {
    let sys = sysinfo::System::new()?;
    let net = sysinfo::Networks::new()?;
    let disks = sysinfo::Disks::new()?;

    let (rx, handle) = monitor::init_background_monitoring(
        sys,
        net,
        disks,
        5,
        std::time::Duration::from_millis(100),
    )?;

    loop {
        match rx.recv() {
            Ok(snapshot) => {
                println!(
                    "CPU Usage: {:.2}%, Memory Usage: {:.2}%, Swap Usage: {:.2}%, Disk Usage: {:.2}%, Network TX: {:.2} bytes, Network RX: {:.2} bytes",
                    snapshot.cpu_usage,
                    snapshot.memory_usage,
                    snapshot.swap_usage,
                    snapshot.disk_usage,
                    snapshot.network_tx,
                    snapshot.network_rx
                );
            }
            Err(e) => {
                eprintln!("Error receiving metrics snapshot: {}", e);
                break;
            }
        }
    }
    handle
        .join()
        .map_err(|e| anyhow::anyhow!("Monitoring thread panicked: {:?}", e))?;
    Ok(())
}

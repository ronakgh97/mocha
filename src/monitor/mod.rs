use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use sysinfo::{CpuRefreshKind, Disks, Networks, System};

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub cpu_usage: f32,
    pub cpu_freq: f32,

    pub memory_usage: f32,
    pub disk_usage: f32,

    pub network_tx: f32,
    pub network_rx: f32,
}

/// Initializes background monitoring of system metrics and returns a receiver
/// for the metrics snapshots and a handle to the monitoring thread.
pub fn init_background_monitoring(
    mut sys: System,
    mut net: Networks,
    mut disk: Disks,
    warmup_count: usize,
    refresh_interval: std::time::Duration,
) -> anyhow::Result<(Receiver<MetricsSnapshot>, JoinHandle<()>)> {
    let (mpsc_tx, mpsc_rx) = std::sync::mpsc::channel::<MetricsSnapshot>();

    // warm up before starting the monitoring thread
    for _ in 0..warmup_count {
        sys.refresh_cpu_list(CpuRefreshKind::everything());
        sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage().with_frequency());
        sys.refresh_memory();
        net.refresh(true);
        disk.refresh(true);
    }

    let handle = std::thread::spawn(move || {
        loop {
            // only refresh what needed to reduce overhead
            sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage().with_frequency());
            sys.refresh_memory();
            net.refresh(false);
            disk.refresh(false);

            let cpus = sys.cpus();
            let cpu_count = cpus.len() as f32;
            let cpu_usage = cpus.iter().map(|cpu| cpu.usage()).sum::<f32>() / cpu_count;
            let cpu_freq = cpus.iter().map(|cpu| cpu.frequency()).sum::<u64>() as f32 / cpu_count;

            let memory_usage = if sys.total_memory() > 0 {
                (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0
            } else {
                0.0
            };

            let network_tx = net.iter().map(|(_, data)| data.transmitted()).sum::<u64>() as f32;
            let network_rx = net.iter().map(|(_, data)| data.received()).sum::<u64>() as f32;

            let total = disk.iter().map(|d| d.total_space()).sum::<u64>();
            let available = disk.iter().map(|d| d.available_space()).sum::<u64>();
            let disk_usage = if total > 0 {
                (total.saturating_sub(available) as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            let snapshot = MetricsSnapshot {
                cpu_usage,
                cpu_freq,
                memory_usage,
                disk_usage,
                network_tx,
                network_rx,
            };

            if mpsc_tx.send(snapshot).is_err() {
                break;
            }

            std::thread::sleep(refresh_interval);
        }
    });

    Ok((mpsc_rx, handle))
}

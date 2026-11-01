use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use sysinfo::{CpuRefreshKind, Disks, Networks, System};

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub swap_usage: f32,
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
        sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
        sys.refresh_memory();
        net.refresh(true);
        disk.refresh(true);
    }

    let handle = std::thread::spawn(move || {
        loop {
            // only refresh the things needed
            sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
            sys.refresh_memory();
            net.refresh(false);
            disk.refresh(false);

            // collect metrics
            let cpu_usage = {
                let cpus = sys.cpus();
                let cpu_count = cpus.len();
                if cpu_count > 0 {
                    cpus.iter().map(|cpu| cpu.usage()).sum::<f32>() / cpu_count as f32
                } else {
                    0.0
                }
            };

            let memory_usage = if sys.total_memory() > 0 {
                (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0
            } else {
                0.0
            };

            let swap_usage = if sys.total_swap() > 0 {
                (sys.used_swap() as f32 / sys.total_swap() as f32) * 100.0
            } else {
                0.0
            };

            let (disk_used, disk_total) = disk.iter().fold((0u64, 0u64), |(used, total), d| {
                (
                    used + d.total_space().saturating_sub(d.available_space()),
                    total + d.total_space(),
                )
            });
            let disk_usage = if disk_total > 0 {
                (disk_used as f32 / disk_total as f32) * 100.0
            } else {
                0.0
            };

            let (network_tx, network_rx) = net.iter().fold((0u64, 0u64), |(tx, rx), (_, d)| {
                (tx + d.transmitted(), rx + d.received())
            });

            // send to UI thread for rendering
            let snapshot = MetricsSnapshot {
                cpu_usage,
                memory_usage,
                swap_usage,
                disk_usage,
                network_tx: network_tx as f32,
                network_rx: network_rx as f32,
            };
            if mpsc_tx.send(snapshot).is_err() {
                break;
            }

            std::thread::sleep(refresh_interval);
        }
    });

    Ok((mpsc_rx, handle))
}

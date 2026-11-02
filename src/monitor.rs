use anyhow::Result;
use crossbeam_channel::Receiver;
use crossbeam_channel::unbounded;
use rdev::Button;
use rdev::Key;
use std::collections::HashSet;
use std::sync::{Arc, LazyLock};
use std::thread::JoinHandle;
use std::time::Duration;
use sysinfo::{CpuRefreshKind, Disks, Networks, System};

#[derive(Debug, Clone, Copy)]
pub struct MetricsSnapshot {
    pub uptime: u64,

    pub cpu_usage: f32,
    pub memory_usage: f32,

    pub swap_usage: f32,
    pub disk_usage: f32,

    pub total_network_tx: u64,
    pub total_network_rx: u64,
}

/// Initializes background monitoring of system metrics and returns a receiver
/// for the metrics snapshots and a handle to the monitoring thread.
pub fn start_metrics_monitoring(
    mut sys: System,
    mut net: Networks,
    mut disk: Disks,
    warmup_count: u64,
    refresh_interval: Duration,
) -> Result<(Receiver<MetricsSnapshot>, JoinHandle<Result<()>>)> {
    let (mpsc_tx, mpsc_rx) = unbounded();

    // warm up before starting the monitoring thread
    for _ in 0..warmup_count {
        sys.refresh_cpu_list(CpuRefreshKind::everything());
        sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
        sys.refresh_memory();
        net.refresh(true);
        disk.refresh(true);
    }

    let handle = std::thread::spawn(move || -> Result<()> {
        sys.refresh_all();
        loop {
            // only refresh the things needed
            sys.refresh_cpu_specifics(CpuRefreshKind::nothing().with_usage());
            sys.refresh_memory();
            net.refresh(false);
            disk.refresh(false);

            // collect metrics
            let uptime = System::uptime()?;

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

            let (total_network_tx, total_network_rx) =
                net.iter().fold((0u64, 0u64), |(tx, rx), (_, d)| {
                    (tx + d.total_transmitted(), rx + d.total_received())
                });

            // send to UI thread for rendering
            let snapshot = MetricsSnapshot {
                uptime,
                cpu_usage,
                memory_usage,
                swap_usage,
                disk_usage,
                total_network_tx,
                total_network_rx,
            };
            if mpsc_tx.send(snapshot).is_err() {
                return Err(anyhow::anyhow!(
                    "Failed to send metrics snapshot to UI thread"
                ));
            }

            std::thread::sleep(refresh_interval);
            std::thread::yield_now();
        }
    });

    Ok((mpsc_rx, handle))
}

pub static KEYPRESS_COUNT: LazyLock<Arc<std::sync::atomic::AtomicU64>> =
    LazyLock::new(|| Arc::new(std::sync::atomic::AtomicU64::new(0)));

pub static MOUSECLICK_COUNT: LazyLock<Arc<std::sync::atomic::AtomicU64>> =
    LazyLock::new(|| Arc::new(std::sync::atomic::AtomicU64::new(0)));

#[derive(Debug, PartialEq, Eq)]
enum InputEvent {
    Key,
    Button,
}

#[derive(Default)]
struct InputEventTracker {
    keys_down: HashSet<Key>,
    buttons_down: HashSet<Button>,
}

impl InputEventTracker {
    fn track(&mut self, event: rdev::EventType) -> Option<InputEvent> {
        match event {
            rdev::EventType::KeyPress(key) => self.keys_down.insert(key).then_some(InputEvent::Key),
            rdev::EventType::KeyRelease(key) => {
                self.keys_down.remove(&key);
                None
            }
            rdev::EventType::ButtonPress(button) => self
                .buttons_down
                .insert(button)
                .then_some(InputEvent::Button),
            rdev::EventType::ButtonRelease(button) => {
                self.buttons_down.remove(&button);
                None
            }
            _ => None,
        }
    }
}

pub fn start_keyboard_mouse_monitoring() -> Result<()> {
    std::thread::spawn(|| {
        let mut tracker = InputEventTracker::default();
        if let Err(e) = rdev::listen(move |event| match tracker.track(event.event_type) {
            Some(InputEvent::Key) => {
                KEYPRESS_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            Some(InputEvent::Button) => {
                MOUSECLICK_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            None => {}
        }) {
            eprintln!("Failed to start keyboard/mouse listener: {e:?}");
        }
    });
    Ok(())
}

use crate::monitor::MetricsSnapshot;
use crossbeam_channel::Receiver;
use eframe::egui;
use eframe::egui::{Color32, RichText, Sense, Ui, ViewportCommand, Visuals};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct UI {
    app_name: String,
    app_tagline: String,

    uptime: u64,

    cpu_usage: f32,
    memory_usage: f32,
    swap_usage: f32,
    disk_usage: f32,

    total_network_tx: u64,
    total_network_rx: u64,

    rx_from_bg: Receiver<MetricsSnapshot>,
    shutdown_requested: Arc<AtomicBool>,
}

impl UI {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        rx_from_bg: Receiver<MetricsSnapshot>,
        shutdown_requested: Arc<AtomicBool>,
    ) -> Self {
        let ctx = cc.egui_ctx.clone();
        ctx.request_repaint();
        Self {
            app_name: "mocha☕".to_string(),
            app_tagline: "a sneak peak into your machine.".to_string(),
            uptime: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            swap_usage: 0.0,
            disk_usage: 0.0,

            total_network_tx: 0,
            total_network_rx: 0,
            rx_from_bg,
            shutdown_requested,
        }
    }
}

impl eframe::App for UI {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // check if shutdown has been requested
        if self.shutdown_requested.load(Ordering::Relaxed) {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        // try draining the channel to get the latest metrics snapshot
        while let Ok(snap) = self.rx_from_bg.try_recv() {
            self.uptime = snap.uptime;
            self.cpu_usage = snap.cpu_usage;
            self.memory_usage = snap.memory_usage;
            self.swap_usage = snap.swap_usage;
            self.disk_usage = snap.disk_usage;
            self.total_network_tx = snap.total_network_tx;
            self.total_network_rx = snap.total_network_rx;
        }

        // base theme
        let text = Color32::from_rgb(240, 224, 213);
        let dim = Color32::from_rgb(160, 140, 135);
        let accent_1 = Color32::from_rgb(235, 170, 165);
        let accent_2 = Color32::from_rgb(235, 200, 165);

        // let width = ui.available_width();
        // let height = ui.available_height();
        egui::Frame::default()
            .fill(Color32::from_rgb(35, 29, 31))
            .corner_radius(16.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                // enable dragging the window by clicking anywhere on the window
                if ui
                    .interact(ui.max_rect(), ui.scope_id(), Sense::drag())
                    .drag_started()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
                }

                let ascii = match self.cpu_usage {
                    x if x >= 80.0 => "(• ˋ_ˊ •)?!",
                    x if x >= 60.0 => "(✿◡‿◡✿)",
                    x if x >= 40.0 => "(✿◕‿◕✿)",
                    _ => "(◕‿◕)",
                };

                // app title and tagline
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.app_name)
                            .size(16.0)
                            .color(accent_1)
                            .strong()
                            .monospace(),
                    );
                    ui.add_space(2.0);
                    ui.label(RichText::new(ascii).size(14.0).color(accent_2));
                });
                ui.label(
                    RichText::new(&self.app_tagline)
                        .size(12.0)
                        .color(dim)
                        .weak()
                        .italics(),
                );
                ui.add_space(2.0);
                ui.label(
                    RichText::new(format!("Uptime: {}", fmt_time(self.uptime)))
                        .size(12.0)
                        .color(text)
                        .weak(),
                );
                ui.add_space(4.0);

                // metrics layout
                let metric = |ui: &mut Ui, label: &str, value: String| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(label).size(12.0).color(accent_2).monospace());
                        ui.add_space(2.0);
                        ui.label(RichText::new(value).size(14.0).color(text));
                    });
                    ui.add_space(4.0);
                };

                // display all metrics
                ui.horizontal(|ui| {
                    metric(ui, "CPU", format!("{:.0}%", self.cpu_usage));
                    metric(ui, "MEM", format!("{:.0}%", self.memory_usage));
                });
                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    metric(ui, "SWP", format!("{:.0}%", self.swap_usage));
                    metric(ui, "DSK", format!("{:.0}%", self.disk_usage));
                });
                ui.add_space(3.0);

                ui.horizontal(|ui| {
                    metric(ui, "TX", fmt_bytes(self.total_network_tx));
                    metric(ui, "RX", fmt_bytes(self.total_network_rx));
                });
                ui.add_space(2.0);

                ui.label(
                    RichText::new("brewed with love ♡.")
                        .size(12.0)
                        .color(dim)
                        .weak()
                        .italics(),
                );
            });

        ui.ctx().request_repaint();
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        Color32::from_rgb(35, 29, 31).to_normalized_gamma_f32()
    }
}

#[inline]
fn fmt_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn fmt_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let bytes_f64 = bytes as f64;

    if bytes_f64 >= GB {
        format!("{:.2} GB", bytes_f64 / GB)
    } else if bytes_f64 >= MB {
        format!("{:.2} MB", bytes_f64 / MB)
    } else if bytes_f64 >= KB {
        format!("{:.2} KB", bytes_f64 / KB)
    } else {
        format!("{} B", bytes)
    }
}

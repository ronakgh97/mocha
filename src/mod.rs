use crate::monitor::MetricsSnapshot;
use crossbeam_channel::Receiver;
use eframe::egui;
use eframe::egui::{Color32, RichText, Sense, Ui, ViewportCommand, Visuals};

pub struct UI {
    app_name: String,
    app_tagline: String,

    cpu_usage: f32,
    memory_usage: f32,
    swap_usage: f32,
    disk_usage: f32,

    network_tx: f32, // in Mbps
    network_rx: f32, // in Mbps
    rx_from_bg: Receiver<MetricsSnapshot>,
}

impl UI {
    pub fn new(cc: &eframe::CreationContext<'_>, rx: Receiver<MetricsSnapshot>) -> Self {
        let ctx = cc.egui_ctx.clone();
        ctx.request_repaint();
        Self {
            app_name: "mocha ☕".to_string(),
            app_tagline: "a sneak peak into your machine.".to_string(),

            cpu_usage: 0.0,
            memory_usage: 0.0,
            swap_usage: 0.0,
            disk_usage: 0.0,

            network_tx: 0.0,
            network_rx: 0.0,
            rx_from_bg: rx,
        }
    }
}

impl eframe::App for UI {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        while let Ok(snap) = self.rx_from_bg.try_recv() {
            self.cpu_usage = snap.cpu_usage;
            self.memory_usage = snap.memory_usage;
            self.swap_usage = snap.swap_usage;
            self.disk_usage = snap.disk_usage;
            self.network_tx = snap.network_tx;
            self.network_rx = snap.network_rx;
        }

        let text = Color32::from_rgb(220, 220, 220);
        let dim = Color32::from_rgb(130, 130, 130);

        egui::Frame::NONE
            .fill(Color32::from_rgb(24, 24, 28))
            .corner_radius(12.0)
            .inner_margin(0.0)
            .show(ui, |ui| {
                // enable dragging of the window by clicking anywhere on the UI
                if ui
                    .interact(ui.max_rect(), ui.scope_id(), Sense::drag())
                    .dragged()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
                }

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.app_name)
                            .size(16.0)
                            .color(text)
                            .strong(),
                    );
                    ui.label(RichText::new(&self.app_tagline).size(11.0).color(dim));
                });

                ui.add_space(12.0);

                let metric = |ui: &mut Ui, label: &str, value: String, color: Color32| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(label).color(dim).size(11.0));
                        ui.label(RichText::new(value).color(color).size(14.0));
                    });
                    ui.add_space(12.0);
                };

                // render the UI
                metric(ui, "CPU", format!("{:.0}%", self.cpu_usage), text);
                metric(ui, "MEM", format!("{:.0}%", self.memory_usage), text);
                metric(ui, "SWP", format!("{:.0}%", self.swap_usage), text);
                metric(ui, "DSK", format!("{:.0}%", self.disk_usage), text);
                metric(ui, "TX ↗", format!("{:.1}M", self.network_tx), dim);
                metric(ui, "RX ↙", format!("{:.1}M", self.network_rx), text);
            });

        ui.ctx().request_repaint();
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}

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

        let text = Color32::from_rgb(240, 224, 213);
        let dim = Color32::from_rgb(160, 140, 135);
        let accent_1 = Color32::from_rgb(235, 170, 165);
        let accent_2 = Color32::from_rgb(235, 200, 165);

        // let width = ui.available_width();
        // let height = ui.available_height();
        egui::Frame::NONE
            .fill(Color32::from_rgb(35, 29, 31))
            .corner_radius(16.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                // enable dragging the window by clicking anywhere on the window
                if ui
                    .interact(ui.max_rect(), ui.scope_id(), Sense::drag())
                    .drag_started()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
                }

                // app title and tagline
                ui.label(
                    RichText::new(&self.app_name)
                        .size(16.0)
                        .color(accent_1)
                        .strong()
                        .monospace(),
                );
                ui.label(
                    RichText::new(&self.app_tagline)
                        .size(12.0)
                        .color(dim)
                        .weak()
                        .italics(),
                );
                ui.add_space(4.0);

                // metrics layout
                let metric = |ui: &mut Ui, label: &str, value: String| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(label).size(12.0).color(accent_2).monospace());
                        ui.add_space(2.0);
                        ui.label(RichText::new(value).size(14.0).color(text));
                    });
                    ui.add_space(5.0);
                };

                // display all metrics
                ui.horizontal(|ui| {
                    metric(ui, "CPU", format!("{:.0}%", self.cpu_usage));
                    metric(ui, "MEM", format!("{:.0}%", self.memory_usage));
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    metric(ui, "SWP", format!("{:.0}%", self.swap_usage));
                    metric(ui, "DSK", format!("{:.0}%", self.disk_usage));
                });
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    metric(ui, "TX", format!("{:.1} Mbps", self.network_tx));
                    metric(ui, "RX", format!("{:.1} Mbps", self.network_rx));
                });
                ui.add_space(4.0);
            });

        ui.ctx().request_repaint();
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        Color32::from_rgb(35, 29, 31).to_normalized_gamma_f32()
    }
}

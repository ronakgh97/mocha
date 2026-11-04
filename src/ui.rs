use crate::monitor::{KEYPRESS_COUNT, MOUSECLICK_COUNT, MetricsSnapshot};
use crate::theme::MochaTheme;
use crossbeam_channel::Receiver;
use eframe::egui;
use eframe::egui::{Align, Layout, RichText, Sense, Ui, ViewportCommand, Visuals};
use eframe::{App, CreationContext, Frame};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub struct UI {
    app_name: String,
    app_tagline: String,
    app_theme: MochaTheme,
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
        cc: &CreationContext<'_>,
        rx_from_bg: Receiver<MetricsSnapshot>,
        shutdown_requested: Arc<AtomicBool>,
    ) -> Self {
        let ctx = cc.egui_ctx.clone();
        ctx.request_repaint();
        Self {
            app_name: "mocha☕".to_string(),
            app_tagline: "a sneak peak into your machine".to_string(),
            app_theme: MochaTheme::default(),
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

impl App for UI {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
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

        let theme = &self.app_theme;

        egui::Frame::default()
            .fill(theme.background)
            .corner_radius(16.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                // enable dragging the window by clicking anywhere on the window
                if ui
                    .interact(ui.max_rect(), ui.scope_id().with("drag"), Sense::drag())
                    .drag_started()
                {
                    ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
                }

                let ascii = match self.cpu_usage {
                    x if x >= 80.0 => "(•ˋ _ ˊ•)",
                    x if x >= 60.0 => "(✿◡‿◡✿)",
                    x if x >= 40.0 => "(✿◕‿◕✿)",
                    _ => "(◕‿◕)",
                };

                // app title and ASCII mood indicator
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&self.app_name)
                            .size(15.0)
                            .color(theme.caramel)
                            .strong()
                            .monospace(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(ascii).size(13.0).color(theme.rose).strong());
                    });
                });
                ui.label(
                    RichText::new(&self.app_tagline)
                        .size(11.0)
                        .color(theme.text_muted)
                        .weak()
                        .italics(),
                );

                // uptime
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Uptime")
                            .size(11.0)
                            .color(theme.text_secondary)
                            .monospace(),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(fmt_time(self.uptime))
                                .size(11.0)
                                .color(theme.text_primary)
                                .monospace()
                                .small()
                                .underline(),
                        );
                    });
                });
                ui.add_space(2.0);

                // reserve space for each label and value so unit changes cannot move columns
                let metric = |ui: &mut Ui, label: &str, value: String| {
                    ui.add_sized(
                        [42.0, 12.0],
                        egui::Label::new(
                            RichText::new(label)
                                .size(10.0)
                                .color(theme.text_secondary)
                                .monospace()
                                .strong(),
                        )
                        .halign(Align::LEFT),
                    );
                    ui.add_sized(
                        [78.0, 12.0],
                        egui::Label::new(
                            RichText::new(&value)
                                .size(11.0)
                                .color(theme.text_primary)
                                .monospace()
                                .small_raised(),
                        )
                        .halign(Align::RIGHT),
                    );
                };

                egui::Grid::new("metrics")
                    .min_col_width(0.0)
                    .spacing([2.0, 1.0])
                    .show(ui, |ui| {
                        metric(ui, "CPU", format!("{:.0}%", self.cpu_usage));
                        metric(ui, "RAM", format!("{:.0}%", self.memory_usage));
                        ui.end_row();

                        metric(ui, "Swap", format!("{:.0}%", self.swap_usage));
                        metric(ui, "Disk", format!("{:.0}%", self.disk_usage));
                        ui.end_row();

                        metric(ui, "Upload", fmt_bytes(self.total_network_tx));
                        metric(ui, "Download", fmt_bytes(self.total_network_rx));
                        ui.end_row();

                        metric(
                            ui,
                            "Keys mashed",
                            KEYPRESS_COUNT.load(Ordering::Relaxed).to_string(),
                        );
                        metric(
                            ui,
                            "Clicks made",
                            MOUSECLICK_COUNT.load(Ordering::Relaxed).to_string(),
                        );
                        ui.end_row();
                    });
                ui.add_space(4.0);

                ui.label(
                    RichText::new("brewed with love 💖")
                        .size(10.0)
                        .color(theme.text_muted)
                        .weak()
                        .italics(),
                );
            });

        ui.ctx().request_repaint();
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        self.app_theme.background.to_normalized_gamma_f32()
    }
}

#[inline]
fn fmt_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let _seconds = seconds % 60;
    format!("{:02}:{:02}", hours, minutes)
}

#[inline]
fn fmt_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let units = ["KB", "MB", "GB", "TB", "PB", "EB"];
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    // promote values that would round up to 1024.00 in the smaller unit
    while value >= 1023.995 && unit < units.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.2} {}", units[unit])
}

mod monitor;
mod theme;
mod ui;

use anyhow::Result;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn main() -> Result<()> {
    let sys = sysinfo::System::new_all()?;
    let net = sysinfo::Networks::new_with_refreshed_list()?;
    let disks = sysinfo::Disks::new_with_refreshed_list()?;
    let shutdown_requested = Arc::new(AtomicBool::new(false));
    let shutdown_for_handler = Arc::clone(&shutdown_requested);

    ctrlc::set_handler(move || {
        shutdown_for_handler.store(true, Ordering::Relaxed);
    })?;

    let (rx, metrics_handle) = monitor::start_metrics_monitoring(
        sys,
        net,
        disks,
        24,
        std::time::Duration::from_millis(750),
    )?;
    monitor::start_keyboard_mouse_monitoring()?;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(false)
            .with_always_on_top()
            .with_inner_size([225.0, 210.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mocha ☕",
        options,
        Box::new(|cc| Ok(Box::new(ui::Gui::new(cc, rx, shutdown_requested)))),
    )?;

    if let Err(e) = metrics_handle.join() {
        eprintln!("Background monitoring thread panicked: {:?}", e);
    }

    Ok(())
}

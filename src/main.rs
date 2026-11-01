pub mod monitor;
pub mod ui;
use anyhow::Result;

fn main() -> Result<()> {
    let sys = sysinfo::System::new()?;
    let net = sysinfo::Networks::new()?;
    let disks = sysinfo::Disks::new()?;

    let (rx, handle) = monitor::start_background_monitoring(
        sys,
        net,
        disks,
        5,
        std::time::Duration::from_millis(100),
    )?;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(false)
            .with_always_on_top()
            .with_inner_size([260.0, 180.0])
            .with_position([1100.0, 30.0]),
        ..Default::default()
    };

    eframe::run_native("mocha", options, Box::new(|cc| Ok(Box::new(ui::UI::new(cc, rx)))))
        .map_err(|e| anyhow::anyhow!("Failed to run: {}", e))?;

    handle
        .join()
        .map_err(|e| anyhow::anyhow!("Monitor thread panicked: {:?}", e))?;
    Ok(())
}
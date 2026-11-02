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
        12,
        std::time::Duration::from_secs(1),
    )?;

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(false)
            .with_always_on_top()
            .with_inner_size([200.0, 140.0]),
        ..Default::default()
    };

    eframe::run_native(
        "mocha",
        options,
        Box::new(|cc| Ok(Box::new(ui::UI::new(cc, rx)))),
    )?;

    if let Err(e) = handle.join() {
        eprintln!("Error joining background monitoring thread: {:?}", e);
    }
    Ok(())
}

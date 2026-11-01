pub struct UI {
    app_name: String,
    app_tagline: String,

    cpu_usage: f32,
    memory_usage: f32,
    swap_usage: f32,
    disk_usage: f32,

    network_tx: f32,
    network_rx: f32,
}

impl Default for UI {
    fn default() -> Self {
        Self {
            app_name: "mocha ☕".to_string(),
            app_tagline: "a sneak peak into your machine.".to_string(),

            cpu_usage: 0.0,
            memory_usage: 0.0,
            swap_usage: 0.0,
            disk_usage: 0.0,

            network_tx: 0.0,
            network_rx: 0.0,
        }
    }
}

impl UI {
    pub fn update(&self) {}
}

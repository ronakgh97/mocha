use eframe::egui::Color32;

#[allow(unused)]
pub struct MochaTheme {
    pub background: Color32,
    pub surface: Color32,
    pub surface_hover: Color32,
    pub surface_active: Color32,
    pub border: Color32,

    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,

    pub rose: Color32,
    pub caramel: Color32,
    pub lavender: Color32,
    pub matcha: Color32,
}

impl Default for MochaTheme {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(41, 35, 33),
            surface: Color32::from_rgb(59, 48, 45),
            surface_hover: Color32::from_rgb(75, 60, 56),
            surface_active: Color32::from_rgb(87, 69, 65),
            border: Color32::from_rgb(87, 69, 65),

            text_primary: Color32::from_rgb(240, 224, 213),
            text_secondary: Color32::from_rgb(190, 165, 155),
            text_muted: Color32::from_rgb(160, 140, 135),

            rose: Color32::from_rgb(235, 170, 165),
            caramel: Color32::from_rgb(235, 200, 165),
            lavender: Color32::from_rgb(217, 184, 206),
            matcha: Color32::from_rgb(185, 203, 170),
        }
    }
}

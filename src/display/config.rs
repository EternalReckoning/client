#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DisplayConfig {
    pub display_mode: DisplayMode,
    pub field_of_view: f32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayMode {
    Windowed,
    Borderless,
    Fullscreen,
}

impl Default for DisplayConfig {
    fn default() -> DisplayConfig {
        DisplayConfig {
            display_mode: DisplayMode::Windowed,
            field_of_view: 68.0,
        }
    }
}
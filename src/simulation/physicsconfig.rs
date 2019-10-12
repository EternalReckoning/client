#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PhysicsConfig {
    pub gravity: f64,
    pub min_collision_depth: f64,
    pub max_ground_slope: f64,
    pub horisontal_drag: f64,
    pub vertical_drag: f64,
}

impl Default for PhysicsConfig {
    fn default() -> PhysicsConfig {
        PhysicsConfig {
            gravity: 0.48,
            min_collision_depth: 0.001,
            max_ground_slope: 0.2,
            horisontal_drag: 0.25,
            vertical_drag: 0.0,
        }
    }
}
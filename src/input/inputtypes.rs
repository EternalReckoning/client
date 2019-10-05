#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum InputTypes {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct KeyMapConfig {
    pub move_forward: u32,
    pub move_backward: u32,
    pub move_left: u32,
    pub move_right: u32,
    pub move_up: u32,
}

impl Default for KeyMapConfig {
    fn default() -> KeyMapConfig {
        KeyMapConfig {
            move_forward: 17,
            move_backward: 31,
            move_left: 30,
            move_right: 32,
            move_up: 57,
        }
    }
}
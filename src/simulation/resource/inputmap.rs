use crate::input::InputTypes;

#[derive(Default)]
pub struct InputMap {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
}

impl InputMap {
    pub fn set(&mut self, input: InputTypes, value: bool) {
        log::debug!("Input state [{:?}]: {}", input, value);
        let field = match input {
            InputTypes::MoveForward => &mut self.move_forward,
            InputTypes::MoveBackward => &mut self.move_backward,
            InputTypes::MoveLeft => &mut self.move_left,
            InputTypes::MoveRight => &mut self.move_right,
            InputTypes::MoveUp => &mut self.move_up,
        };
        *field = value;
    }
}
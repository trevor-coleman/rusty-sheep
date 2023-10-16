use bevy::prelude::{Input, KeyCode, Res};
use crate::dog::DogCommand;

pub enum GameControl {
    Up,
    Down,
    Left,
    Right,
}

impl GameControl {
    pub fn pressed(&self, keyboard_input: &Res<Input<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up)
            }
            GameControl::Down => {
                keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down)
            }
            GameControl::Left => {
                keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left)
            }
            GameControl::Right => {
                keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right)
            }
        }
    }
}

// pub fn get_movement(control: GameControl, input: &Res<Input<KeyCode>>) -> f32 {
//     if control.pressed(input) {
//         1.0
//     } else {
//         0.0
//     }
// }

pub fn get_command_for_input(input: &Res<Input<KeyCode>>) -> Option<DogCommand> {
    if GameControl::Down.pressed(input) {
        Some(DogCommand::LayDown)
    } else if GameControl::Right.pressed(input) {
        Some(DogCommand::Away)
    } else if GameControl::Left.pressed(input) {
        Some(DogCommand::ComeBye)
    } else if GameControl::Up.pressed(input) {
        Some(DogCommand::WalkOn)
    } else {
        None
    }
}

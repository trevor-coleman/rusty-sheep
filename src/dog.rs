use bevy::ecs::system::assert_system_does_not_conflict;
use bevy::prelude::*;
use crate::actions::Actions;
use crate::dog::DogCommand::{Away, ComeBye, LayDown};

use crate::GameState;
use crate::loading::TextureAssets;

pub struct DogPlugin;

//enum of sheepdog commands
#[derive(Debug, Clone)]
pub enum DogCommand {
    ComeBye,
    Away,
    LayDown,
    WalkOn,
    Easy,
}


#[derive(Component)]
pub struct Dog {
    command: DogCommand
}

impl Dog {
    pub fn new() -> Self {
        Self { command: Away }
    }
    pub fn set_command(&mut self, command: &DogCommand) {
        self.command = command.clone();;
    }


}

impl Plugin for DogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_dog)
            .add_systems(Update, run_in_a_circle.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_dog(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn(SpriteBundle {
            texture: textures.dog.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)).with_scale(Vec3::new(0.3, 0.3, 0.5)),
            ..Default::default()
        }).insert(Dog {command: Away });
}

fn listen_to_commands(
    actions: Res<Actions>,
    mut dog_query: Query<&mut Dog>,
) {

}

fn run_in_a_circle(
    time: Res<Time>,
    mut dog_query: Query<(&mut Transform, &Dog)>,
) {
    let speed = 20.;

    let elapsed_time = time.raw_elapsed_seconds() as f32;
    let sin = elapsed_time.sin() * 0.1;
    let cos = elapsed_time.cos() * 0.1;


    let mut movement = Vec3::new(
        &speed * sin,
        &speed * cos,
        0.,
    );

    for (mut transform, dog) in &mut dog_query {
        match &dog.command {
            LayDown => {
                movement = Vec3::new(0., 0., 0.);
            }
            Away => {
                transform.translation += Vec3::new(
                    &speed * cos,
                    &speed * sin,
                    0.,
                );
            }
            ComeBye => {
                transform.translation += Vec3::new(
                    &speed * sin,
                    &speed * cos,
                    0.,
                );
            }
            _ => {}
        }
    }

}

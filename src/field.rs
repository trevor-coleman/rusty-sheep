use bevy::prelude::*;

use crate::GameState;
use crate::loading::TextureAssets;

pub struct FieldPlugin;

#[derive(Resource)]
pub struct Field {
    pub width: f32,
    pub height: f32,
    pub origin: (f32, f32),
    pub sprites: Vec<Entity>,
}

impl Field {
    pub fn new(width: f32, height: f32) -> Self {
        let origin = (&width / -2.0, &height / 2.0);
        Self { width, height, origin, sprites: vec![] }
    }

    pub fn update_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.origin = (&width / -2.0, &height / 2.0);
    }

    pub fn width_in_tiles(&self, tile_width: Option<f32>) -> i32 {
        let tile_width = match tile_width {
            Some(th) => th,
            None => 64.
        };
        ((self.width.clone() / tile_width).ceil() + 1.) as i32

    }

    pub fn height_in_tiles(&self, tile_height: Option<f32>) -> i32 {
        let tile_height = match tile_height {
            Some(th) => th,
            None => 64.
        };
        ((self.height.clone() / tile_height).ceil() + 1.) as i32

    }

    pub fn tile_offset(&self, tile_width: Option<f32>, tile_height: Option<f32>) -> (f32, f32) {
        let tile_height = match tile_height {
            Some(th) => th,
            None => 64.
        };
        let tile_width = match tile_width {
            Some(tw) => tw,
            None => 64.
        };

        let overhang_x = (self.width_in_tiles(Some(tile_width)) as f32)
            * &tile_width
            - &self.width;
        let overhang_y = (self.height_in_tiles(Some(tile_height)) as f32)
            * &tile_height
            - &self.height;

        (overhang_x / -2., overhang_y / -2.)
    }

    pub fn tile_origin(&self, tile_width: Option<f32>, tile_height: Option<f32>) -> (f32, f32) {
        let (x, y) = self.tile_offset(tile_width, tile_height);
        (&self.origin.0 + x, &self.origin.1 - y)
    }

    pub fn despawn_tiles(&mut self, commands: &mut Commands) {
        for sprite in self.sprites.iter() {
            commands.entity(*sprite).despawn();
        }
        self.sprites.clear();
    }

    pub fn spawn_tiles(&mut self, commands: &mut Commands, textures: &Res<TextureAssets>) {

        &self.despawn_tiles(commands);

        let (x, y) = self.tile_origin(None, None);
        let mut x = x;
        let mut y = y;

        let mut i = 0;
        while i < self.width_in_tiles(None) {
            let mut j = 0;
            while j < self.height_in_tiles(None) {
                self.sprites.push(commands.spawn(
                    SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(x.clone() , y.clone(), 0.)),
                        texture: textures.texture_tile_grass_1.clone(),
                        ..Default::default()
                    }
                ).id());
                y -= 64.;
                j += 1;
            }
            x += 64.;
            y = self.tile_origin(None, None).1;
            i += 1;
        }
    }
}

impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Field::new(0.0, 0.0))
            .add_systems(Update, spawn_field.run_if(in_state(GameState::Playing)));

    }
}

fn spawn_field(mut commands: Commands, textures: Res<TextureAssets>, mut windows: Query<&mut Window>, mut field: ResMut<Field>) {
    let window = windows.iter_mut().next().unwrap();

    if !window.is_changed() {
        return;
    }

    println!("Window changed: width: {}, height: {}", window.width(), window.height());

    field.update_size(window.width(), window.height());

    println!("Field size in tiles: width: {}, height: {}", field.width_in_tiles(None), field.height_in_tiles(None));

    field.spawn_tiles(&mut commands, &textures);
}

fn find_bottom_right(window: &Window) -> Vec3 {
    Vec3::new(window.width() / 2.0, window.height() / -2.0, 0.0)
}



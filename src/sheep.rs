use bevy::prelude::*;
use rand::random;

use crate::field::Field;
use crate::GameState;
use crate::loading::TextureAssets;

pub struct SheepPlugin;


#[derive(Component, Debug)]
pub struct Sheep {
    id: i32,
    vx: f32,
    vy: f32,
    close_dx: f32,
    close_dy: f32,
    xvel_avg: f32,
    yvel_avg: f32,
    num_neighbors: i32,
    xpos_avg: f32,
    ypos_avg: f32,
    bounced_x: bool,
    bounced_y: bool,
    bias_x: f32,
    bias_y: f32,
}

impl Sheep {
    pub fn new() -> Self {
        let random_x: f32 = -0.5 + random::<f32>();
        let random_y: f32 = -0.5 + random::<f32>();
        Self {
            id: 0,
            vx: random_x * 100.0,
            vy: random_y * 100.0,
            close_dy: 0.0,
            xvel_avg: 0.0,
            yvel_avg: 0.0,
            num_neighbors: 0,
            xpos_avg: 0.0,
            close_dx: 0.0,
            ypos_avg: 0.0,
            bounced_x: false,
            bounced_y: false,
            bias_x: 0.0,
            bias_y: 0.0,
        }
    }
}


impl Plugin for SheepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_sheep)
            // .add_systems(Update, flock.before(move_sheep).run_if(in_state(GameState::Playing)))
            .add_systems(Update, move_and_flock_sheep.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_sheep(mut commands: Commands, textures: Res<TextureAssets>, field: Res<Field>) {

    const BIAS_STRENGTH: f32 = 0.02;

    let bias_a_x = (-0.5  + random::<f32>()) * BIAS_STRENGTH;
    let bias_a_y = (-0.5  + random::<f32>()) * BIAS_STRENGTH;
    let bias_b_x = (-0.5  + random::<f32>()) * BIAS_STRENGTH;
    let bias_b_y = (-0.5  + random::<f32>()) * BIAS_STRENGTH;

    for i in 1..45 {
        let rand_vel_x: f32 = -0.5 + random::<f32>();
        let rand_vel_y: f32 = -0.5 + random::<f32>();
        let rand_pos_y: f32 = -0.5 + random::<f32>();
        let rand_pos_y: f32 = -0.5 + random::<f32>();
        commands
            .spawn(SpriteBundle {
                texture: textures.sheep.clone(),
                transform: Transform::from_translation(Vec3::new(rand_vel_x * 1000.0, rand_pos_y * 1000.0, 0.0)).with_scale(Vec3::new(0.1, 0.1, 0.1)),
                ..Default::default()
            }).insert(Sheep {
            id: i,
            vx: rand_vel_x * 10.0,
            vy: rand_vel_y * 10.0,
            close_dy: 0.0,
            xvel_avg: 0.0,
            yvel_avg: 0.0,
            close_dx: 0.0,
            num_neighbors: 0,
            xpos_avg: 0.0,
            ypos_avg: 0.0,
            bounced_x: false,
            bounced_y: false,
            bias_x: match i % 8 {
                0 => {
                    bias_a_x
                }
                1 => {
                    bias_b_x
                }
                _ => { 0.0}
            },
            bias_y: match i % 3 {
                0 => {
                    bias_a_y
                }
                1 => {
                    bias_b_y
                }
                _ => { 0.0}
            }
        });
    }
}


fn move_and_flock_sheep(
    field: ResMut<Field>,
    time: Res<Time>,
    mut sheep_query: Query<(&mut Transform, &mut Sheep)>,
) {
    const MAX_SPEED: f32 = 50.0;
    const BOUNDARY_DAMPING: f32 = 2.0;
    const SPEED: f32 = 150.0;

    const PROTECTED_DISTANCE: f32 = 40.0;
    const VISIBLE_DISTANCE: f32 = 200.0;
    const AVOID_FACTOR: f32 = 0.008;
    const ALIGN_FACTOR: f32 = 0.18;
    const CENTERING_FACTOR: f32 = 0.000001;

    // Initialize field boundaries
    let max_x = 0.9 * field.width / 2.0;
    let max_y = 0.9 * field.height / 2.0;
    let min_x = -0.9 * field.width / 2.0;
    let min_y = -0.9 * field.height / 2.0;

    // Reset sheep attributes
    for (_, mut sheep) in sheep_query.iter_mut() {
        sheep.close_dx = 0.0;
        sheep.close_dy = 0.0;
        sheep.xvel_avg = 0.0;
        sheep.yvel_avg = 0.0;
        sheep.xpos_avg = 0.0;
        sheep.ypos_avg = 0.0;
        sheep.num_neighbors = 0;
    }



    let mut combinations = sheep_query.iter_combinations_mut();

    while let Some([(mut a_transform, mut a_sheep), (mut b_transform, mut b_sheep)]) = combinations.fetch_next() {
        let dx = a_transform.translation.x - b_transform.translation.x;
        let dy = a_transform.translation.y - b_transform.translation.y;
        let distance = (dx * dx + dy * dy).sqrt();


        if distance <= PROTECTED_DISTANCE {
            // avoid
            a_sheep.close_dx += dx;
            a_sheep.close_dy += dy;
            b_sheep.close_dx -= dx;
            b_sheep.close_dy -= dy;
        } else if distance <= VISIBLE_DISTANCE {

            // align and cohere
            a_sheep.num_neighbors += 1;
            b_sheep.num_neighbors += 1;

            a_sheep.xvel_avg += b_sheep.vx;
            a_sheep.yvel_avg += b_sheep.vy;
            b_sheep.xvel_avg += a_sheep.vx;
            b_sheep.yvel_avg += a_sheep.vy;

            a_sheep.xpos_avg += b_transform.translation.x;
            a_sheep.ypos_avg += b_transform.translation.y;
            b_sheep.xpos_avg += a_transform.translation.x;
            b_sheep.ypos_avg += a_transform.translation.y;
        }
    }

    for (mut transform, mut sheep) in sheep_query.iter_mut() {
        let mut adjustment_x = 0.0;
        let mut adjustment_y = 0.0;

        if sheep.num_neighbors > 0 {
            let xvel_avg = sheep.xvel_avg / sheep.num_neighbors as f32;
            let yvel_avg = sheep.yvel_avg / sheep.num_neighbors as f32;

            let xpos_avg = sheep.xpos_avg / sheep.num_neighbors as f32;
            let ypos_avg = sheep.ypos_avg / sheep.num_neighbors as f32;


            // align

            let align_x = xvel_avg - sheep.vx;
            let align_y = yvel_avg - sheep.vy;


            adjustment_x += align_x * ALIGN_FACTOR;
            adjustment_y += align_y * ALIGN_FACTOR;


            // centering

            let center_x = xpos_avg - transform.translation.x;
            let center_y = ypos_avg - transform.translation.y;


            adjustment_x += center_x * CENTERING_FACTOR;
            adjustment_y += center_y * CENTERING_FACTOR;
        } else {}

        // avoid
        adjustment_x += sheep.close_dx * AVOID_FACTOR;
        adjustment_y += sheep.close_dy * AVOID_FACTOR;

        adjustment_x += sheep.bias_x;
        adjustment_y += sheep.bias_y;

        println!("sheep x/y ({}, {})", transform.translation.x, transform.translation.y);
        println!("adj x/y   ({}, {})", adjustment_x, adjustment_y);

        sheep.vx = clamp_velocity(sheep.vx, MAX_SPEED);
        sheep.vy = clamp_velocity(sheep.vy, MAX_SPEED);


        sheep.vx = sheep.vx + adjustment_x;
        sheep.vy = sheep.vy + adjustment_y;

        sheep.vx = clamp_velocity(sheep.vx, MAX_SPEED);
        sheep.vy = clamp_velocity(sheep.vy, MAX_SPEED);

        // Limit velocity

    }

    // Movement logic
    for (mut sheep_transform, mut sheep) in &mut sheep_query {
        // Calculate new position based on current velocity
        let new_x = sheep_transform.translation.x + sheep.vx * SPEED * time.delta_seconds();
        let new_y = sheep_transform.translation.y + sheep.vy * SPEED * time.delta_seconds();

        // Boundary checks and gradual damping
        if new_x >= max_x {
            sheep.vx -= BOUNDARY_DAMPING;
        } else if new_x <= min_x {
            sheep.vx += BOUNDARY_DAMPING;
        }

        if new_y >= max_y {
            sheep.vy -= BOUNDARY_DAMPING;
        } else if new_y <= min_y {
            sheep.vy += BOUNDARY_DAMPING;
        }

        // Calculate new position based on adjusted velocity
        let adjusted_x = sheep_transform.translation.x + sheep.vx * SPEED * time.delta_seconds();
        let adjusted_y = sheep_transform.translation.y + sheep.vy * SPEED * time.delta_seconds();

        // Update position
        sheep_transform.translation.x = adjusted_x;
        sheep_transform.translation.y = adjusted_y;
    }
}


fn clamp_velocity(velocity: f32, max_speed: f32) -> f32 {
    if velocity > max_speed {
        return max_speed;
    } else if velocity < -max_speed {
        return -max_speed;
    }
    velocity
}
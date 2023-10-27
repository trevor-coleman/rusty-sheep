use bevy::prelude::*;
use rand::{random, Rng};

use crate::field::Field;
use crate::GameState;
use crate::loading::TextureAssets;
use bevy::math::Vec2;


pub struct SheepPlugin;


#[derive(Component, Debug)]
pub struct Sheep {
    id: i32,
    velocity: Vec2,
    close_d: Vec2,
    vel_avg: Vec2,
    num_neighbors: i32,
    pos_avg: Vec2,
    bounced: (bool, bool),
    bias: Vec2,
}

impl Sheep {
    pub fn new() -> Self {
        let random_velocity = Vec2::new(
            (-0.5 + random::<f32>()) * 100.0,
            (-0.5 + random::<f32>()) * 100.0,
        );

        Self {
            id: 0,
            velocity: random_velocity,
            close_d: Vec2::ZERO,
            vel_avg: Vec2::ZERO,
            num_neighbors: 0,
            pos_avg: Vec2::ZERO,
            bounced: (false, false),
            bias: Vec2::ZERO,
        }
    }

    pub fn clear(&mut self) {
        self.close_d = Vec2::ZERO;
        self.vel_avg = Vec2::ZERO;
        self.pos_avg = Vec2::ZERO;
        self.num_neighbors = 0;
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
    const BIAS_STRENGTH: f32 = 0.008;

    let bias_a = Vec2::new(
        (-0.5 + random::<f32>()) * BIAS_STRENGTH,
        (-0.5 + random::<f32>()) * BIAS_STRENGTH,
    );
    let bias_b = Vec2::new(
        (-0.5 + random::<f32>()) * BIAS_STRENGTH,
        (-0.5 + random::<f32>()) * BIAS_STRENGTH,
    );

    for i in 1..200 {
        let rand_vel = Vec2::new(
            (-0.5 + random::<f32>()) * 10.0,
            (-0.5 + random::<f32>()) * 10.0,
        );
        let rand_pos = Vec2::new(
            (-0.5 + random::<f32>()) * 1000.0,
            (-0.5 + random::<f32>()) * 1000.0,
        );

        commands
            .spawn(SpriteBundle {
                texture: textures.sheep.clone(),
                transform: Transform::from_translation(Vec3::new(rand_pos.x, rand_pos.y, 0.0)).with_scale(Vec3::new(0.1, 0.1, 0.1)),
                ..Default::default()
            })
            .insert(Sheep {
                id: i,
                velocity: rand_vel,
                close_d: Vec2::ZERO,
                vel_avg: Vec2::ZERO,
                num_neighbors: 0,
                pos_avg: Vec2::ZERO,
                bounced: (false, false),
                bias: match i % 8 {
                    0 => bias_a,
                    1 => bias_b,
                    _ => Vec2::ZERO,
                },
            });
    }
}

// Calculates interaction between a pair of sheep for alignment and cohesion
fn calculate_pair_interaction(
    a_transform: &mut Transform,
    a_sheep: &mut Sheep,
    b_transform: &mut Transform,
    b_sheep: &mut Sheep,
    protected_distance: f32,
    visible_distance: f32,
) {
    let d = (a_transform.translation - b_transform.translation).truncate();
    let distance = d.length();

    if distance <= protected_distance {
        // Avoid
        a_sheep.close_d += d;
        b_sheep.close_d -= d;
    } else if distance <= visible_distance {
        // Align and Cohere
        a_sheep.num_neighbors += 1;
        b_sheep.num_neighbors += 1;

        a_sheep.vel_avg += b_sheep.velocity;
        b_sheep.vel_avg += a_sheep.velocity;

        a_sheep.pos_avg += b_transform.translation.truncate();
        b_sheep.pos_avg += a_transform.translation.truncate();
    }
}

// Modifies the velocity of a single sheep based on flocking rules
fn apply_flocking_rule_for_single_sheep(
    sheep_transform: &mut Transform,
    sheep: &mut Sheep,
    align_factor: f32,
    centering_factor: f32,
    avoid_factor: f32,
    max_speed: f32,
) {
    let mut adjustment = Vec2::ZERO;

    if sheep.num_neighbors > 0 {
        // Alignment and Centering adjustments
        let vel_avg = sheep.vel_avg / sheep.num_neighbors as f32;
        let pos_avg = sheep.pos_avg / sheep.num_neighbors as f32;

        let align = vel_avg - sheep.velocity;
        adjustment += align * align_factor;

        let center = pos_avg - sheep_transform.translation.truncate();
        adjustment += center * centering_factor;
    }

    // Avoidance and Bias adjustments
    adjustment += sheep.close_d * avoid_factor;
    adjustment += sheep.bias;

    // Apply adjustments and clamp the velocity
    sheep.velocity += adjustment;
    sheep.velocity = clamp_velocity(sheep.velocity, max_speed);  // Assume you've implemented clamp_velocity
    wander(sheep);
}

fn move_and_flock_sheep(
    field: ResMut<Field>,
    time: Res<Time>,
    mut sheep_query: Query<(&mut Transform, &mut Sheep)>,
) {
    const MAX_SPEED: f32 = 2.0;
    const BOUNDARY_DAMPING: f32 = 0.1;
    const SPEED: f32 = 150.0;

    const PROTECTED_DISTANCE: f32 = 50.0;
    const VISIBLE_DISTANCE: f32 = 100.0;
    const AVOID_FACTOR: f32 = 0.002;
    const ALIGN_FACTOR: f32 = 0.01;
    const CENTERING_FACTOR: f32 = 0.0001;

    // Initialize field boundaries
    let max_x = 0.8 * field.width / 2.0;
    let max_y = 0.8 * field.height / 2.0;
    let min_x = -0.8 * field.width / 2.0;
    let min_y = -0.8 * field.height / 2.0;

    // Reset sheep attributes
    for (_, mut sheep) in sheep_query.iter_mut() {
        sheep.clear();
    }


    let mut combinations = sheep_query.iter_combinations_mut();

    while let Some([(mut a_transform, mut a_sheep), (mut b_transform, mut b_sheep)]) = combinations.fetch_next() {
        calculate_pair_interaction(&mut a_transform, &mut a_sheep, &mut b_transform, &mut b_sheep, PROTECTED_DISTANCE, VISIBLE_DISTANCE);
    }

    for (mut transform, mut sheep) in sheep_query.iter_mut() {
        apply_flocking_rule_for_single_sheep(&mut transform, &mut sheep, ALIGN_FACTOR, CENTERING_FACTOR, AVOID_FACTOR, MAX_SPEED);
    }

    // Movement logic
    for (mut sheep_transform, mut sheep) in &mut sheep_query {
        // Calculate new position based on current velocity

        let new_position = Vec2 {
            x: sheep_transform.translation.x + sheep.velocity.x * SPEED * time.delta_seconds(),
            y: sheep_transform.translation.y + sheep.velocity.y * SPEED * time.delta_seconds(),
        };

        // Boundary checks and gradual damping
        if new_position.x >= max_x {
            sheep.velocity.x -= BOUNDARY_DAMPING;
        } else if new_position.x <= min_x {
            sheep.velocity.x += BOUNDARY_DAMPING;
        }

        if new_position.y >= max_y {
            sheep.velocity.y -= BOUNDARY_DAMPING;
        } else if new_position.y <= min_y {
            sheep.velocity.y += BOUNDARY_DAMPING;
        }

        // Calculate new position based on adjusted velocity
        let adjusted = Vec2 {
            x: sheep_transform.translation.x + sheep.velocity.x * SPEED * time.delta_seconds(),
            y: sheep_transform.translation.y + sheep.velocity.y * SPEED * time.delta_seconds(),
        };


        // Update position
        sheep_transform.translation = adjusted.extend(0.0);
    }
}


fn clamp_velocity(velocity: Vec2, max_speed: f32) -> Vec2 {
    let mut velocity = velocity.clone();

    if velocity.x > max_speed {
        velocity.x = max_speed;
    } else if velocity.x < -max_speed {
        velocity.x = -max_speed;
    }

    if velocity.y > max_speed {
        velocity.y = max_speed;
    } else if velocity.y < -max_speed {
        velocity.y = -max_speed;
    }

    velocity
}

/// Produces a random unit vector.
fn random_unit_vector() -> Vec2 {
    let angle: f32 = rand::thread_rng().gen_range(0.0..std::f32::consts::TAU); // TAU is 2*PI
    Vec2::new(angle.cos(), angle.sin())
}

fn wander(sheep: &mut Sheep) {
    const WANDER_FORCE: f32 = 0.1;
    // Some method to produce a random unit vector
    let random_dir = random_unit_vector();
    sheep.velocity += random_dir * WANDER_FORCE;
}
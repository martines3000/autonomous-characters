use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

use bevy::{math::const_vec2, prelude::*, tasks::TaskPool};
use bevy_prototype_lyon::prelude::*;

use rand::prelude::*;

use crate::{world::WALL_MARGIN, MainCamera};

const VEHICLE_COUNT: usize = 100;
const VEHICLE_SIZE: f32 = 10.0;
const VEHICLE_MAX_SPEED: f32 = 300.0;
const VEHICLE_MAX_SPEED_VEC: Vec2 = const_vec2!([VEHICLE_MAX_SPEED; 2]);
const VEHICLE_MAX_FORCE: Vec2 = const_vec2!([80.0; 2]);
const VEHICLE_MASS: f32 = 10.0;
const VEHICLE_BODY_COLOR: Color = Color::WHITE;
const VEHICLE_EDGE_COLOR: Color = Color::BLUE;
const VEHICLE_WANDER_SPEED: f32 = 150.0;
const VEHICLE_PREDICT_DISTANCE: f32 = 300.0;
const VEHICLE_PREDICT_RADIUS: f32 = 100.0;

// Distances
const VEHICLE_SEPERATION_DIST: f32 = VEHICLE_SIZE * 2.0;
const VEHICLE_ALIGN_DIST: f32 = VEHICLE_SIZE * 6.0;
const VEHICLE_COHESION_DIST: f32 = VEHICLE_SIZE * 6.0;
const VEHICLE_SEPERATION_DIST_SQ: f32 = VEHICLE_SEPERATION_DIST * VEHICLE_SEPERATION_DIST;
const VEHICLE_ALIGN_DIST_SQ: f32 = VEHICLE_ALIGN_DIST * VEHICLE_ALIGN_DIST;
const VEHICLE_COHESION_DIST_SQ: f32 = VEHICLE_COHESION_DIST * VEHICLE_COHESION_DIST;

// Force factors
const VEHICLE_SEPERATION_FACTOR: f32 = 1.8;
const VEHICLE_ALIGN_FACTOR: f32 = 1.2;
const VEHICLE_COHESION_FACTOR: f32 = 0.6;
const VEHICLE_LIMIT_FACTOR: f32 = 1.6;
const VEHICLE_TARGET_FACTOR: f32 = 1.4;
const VEHICLE_WANDER_FACTOR: f32 = 1.0;

const LINE_WIDTH: f32 = 2.0;
pub const TARGET_RADIUS: f32 = 100.0;

pub struct VehiclePlugin;

#[derive(Component)]
pub struct Vehicle;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Deref, DerefMut)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct WanderTheta(f32);

impl Acceleration {
    fn apply_force(&mut self, force: Vec2, mass: &Mass) {
        self.0 += force / mass.0;
    }
}

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_vehicles)
            .add_system(calc_movement)
            .add_system(update);
    }
}

fn spawn_vehicles(mut commands: Commands, windows: Res<Windows>) {
    let mut rng = rand::thread_rng();
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width() as f32, window.height() as f32) / 2.0 - WALL_MARGIN;

    for i in 0..=VEHICLE_COUNT {
        let x = rng.gen_range(-window_size.x..window_size.x);
        let y = rng.gen_range(-window_size.y..window_size.y);

        let shape = shapes::RegularPolygon {
            sides: 3,
            feature: shapes::RegularPolygonFeature::Radius(VEHICLE_SIZE),
            ..shapes::RegularPolygon::default()
        };

        let line = shapes::Line {
            0: shape.center,
            1: shape.center + Vec2::new(0.0, VEHICLE_SIZE),
        };

        let builder = GeometryBuilder::new().add(&shape).add(&line);

        commands
            .spawn_bundle(builder.build(
                DrawMode::Outlined {
                    fill_mode: {
                        bevy_prototype_lyon::draw::FillMode {
                            options: FillOptions::non_zero(),
                            color: VEHICLE_BODY_COLOR,
                        }
                    },
                    outline_mode: StrokeMode::new(VEHICLE_EDGE_COLOR, LINE_WIDTH),
                },
                Transform {
                    translation: Vec3::new(x, y, 900.0),
                    ..Default::default()
                },
            ))
            .insert(Vehicle)
            .insert(Velocity(Vec2::ZERO))
            .insert(Acceleration(Vec2::ZERO))
            .insert(Mass(VEHICLE_MASS))
            .insert(WanderTheta(0.0))
            .insert(Name::new(format!("{}_{}", "Vehicle", i)));
    }
}

fn seek_steer(world_pos: &Vec2, transform: &Transform, desired: &mut Vec2) {
    *desired = *world_pos - transform.translation.truncate();

    let dist = desired.length();

    *desired = desired.normalize_or_zero();

    if dist < TARGET_RADIUS {
        *desired *= dist / TARGET_RADIUS * VEHICLE_MAX_SPEED;
    } else {
        *desired *= VEHICLE_MAX_SPEED;
    }
}

fn flock(
    acceleration: &mut Acceleration,
    transform: &Transform,
    velocity: &Velocity,
    mass: &Mass,
    other_vehicle_query: &Query<(&Transform, &Velocity), With<Vehicle>>,
    task_pool: &Res<TaskPool>,
) {
    // Seperate
    let seperate_sum = Arc::new(Mutex::new(Vec2::new(0.0, 0.0)));
    let seperate_count = Arc::new(Mutex::new(0));

    // Align
    let align_sum = Arc::new(Mutex::new(Vec2::new(0.0, 0.0)));
    let align_count = Arc::new(Mutex::new(0));

    // Cohesion
    let cohesion_sum = Arc::new(Mutex::new(Vec2::new(0.0, 0.0)));
    let cohesion_count = Arc::new(Mutex::new(0));

    // println!("{}", task_pool.thread_num());

    other_vehicle_query.par_for_each(&task_pool, 500, |(other_transform, other_velocity)| {
        let dist = transform
            .translation
            .truncate()
            .distance_squared(other_transform.translation.truncate());

        // Seperate
        if dist > 0.0 {
            if dist <= VEHICLE_SEPERATION_DIST_SQ {
                let mut seperate_sum_lock = seperate_sum.lock().unwrap();
                *seperate_sum_lock += (transform.translation.truncate()
                    - other_transform.translation.truncate())
                .normalize_or_zero()
                    / dist.sqrt();

                let mut seperate_count_lock = seperate_count.lock().unwrap();
                *seperate_count_lock += 1;
            }

            // Align
            if dist <= VEHICLE_ALIGN_DIST_SQ {
                let mut align_sum_lock = align_sum.lock().unwrap();
                *align_sum_lock += other_velocity.0;

                let mut align_count_lock = align_count.lock().unwrap();
                *align_count_lock += 1;
            }

            // Cohesion
            if dist <= VEHICLE_COHESION_DIST_SQ {
                let mut cohesion_sum_lock = cohesion_sum.lock().unwrap();
                *cohesion_sum_lock += other_transform.translation.truncate();

                let mut cohesion_count_lock = cohesion_count.lock().unwrap();
                *cohesion_count_lock += 1;
            }
        }
    });

    // Seperate
    let seperate_count_lock = seperate_count.lock().unwrap();
    let mut seperate_sum_lock = seperate_sum.lock().unwrap();

    if *seperate_count_lock > 0 {
        *seperate_sum_lock /= *seperate_count_lock as f32;
        *seperate_sum_lock = seperate_sum_lock.normalize_or_zero() * VEHICLE_MAX_SPEED;

        acceleration.apply_force(
            (*seperate_sum_lock - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_SEPERATION_FACTOR,
            mass,
        );
    }

    // Align
    let align_count_lock = align_count.lock().unwrap();
    let mut align_sum_lock = align_sum.lock().unwrap();

    if *align_count_lock > 0 {
        *align_sum_lock /= *align_count_lock as f32;
        *align_sum_lock = align_sum_lock.normalize_or_zero() * VEHICLE_MAX_SPEED;

        acceleration.apply_force(
            (*align_sum_lock - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_ALIGN_FACTOR,
            mass,
        );
    }

    // Cohesion
    let cohesion_count_lock = cohesion_count.lock().unwrap();
    let mut cohesion_sum_lock = cohesion_sum.lock().unwrap();

    if *cohesion_count_lock > 0 {
        let mut desired = Vec2::ZERO;
        *cohesion_sum_lock /= *cohesion_count_lock as f32;

        seek_steer(&cohesion_sum_lock, &transform, &mut desired);

        acceleration.apply_force(
            (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_COHESION_FACTOR,
            mass,
        );
    }
}

fn calc_movement(
    mut vehicle_query: Query<
        (
            &Velocity,
            &Transform,
            &mut Acceleration,
            &Mass,
            &mut WanderTheta,
        ),
        With<Vehicle>,
    >,
    other_vehicle_query: Query<(&Transform, &Velocity), With<Vehicle>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    task_pool: Res<TaskPool>,
) {
    let window = windows.get_primary().unwrap();
    let (camera, camera_transform) = camera_query.single();
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

    let mut wander = true;

    if let Some(_position) = window.cursor_position() {
        let ndc = (_position / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        let world_pos: Vec2 = world_pos.truncate();

        // Folow mouse position
        if buttons.pressed(MouseButton::Left) {
            vehicle_query.par_for_each_mut(
                &task_pool,
                500,
                |(velocity, transform, mut acceleration, mass, _)| {
                    let mut desired = Vec2::ZERO;
                    seek_steer(&world_pos, &transform, &mut desired);

                    acceleration.apply_force(
                        (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                            * VEHICLE_TARGET_FACTOR,
                        mass,
                    );

                    flock(
                        &mut acceleration,
                        &transform,
                        &velocity,
                        mass,
                        &other_vehicle_query,
                        &task_pool,
                    );
                },
            );

            wander = false;
        }
    }

    let bounds = window_size / 2.0 - WALL_MARGIN;

    // Wander
    if wander {
        let range = PI / 8.0;

        vehicle_query.par_for_each_mut(
            &task_pool,
            500,
            |(velocity, transform, mut acceleration, mass, mut wander_theta)| {
                let fx = transform.translation.x < -bounds.x || transform.translation.x > bounds.x;
                let fy = transform.translation.y < -bounds.y || transform.translation.y > bounds.y;

                flock(
                    &mut acceleration,
                    &transform,
                    &velocity,
                    mass,
                    &other_vehicle_query,
                    &task_pool,
                );

                if !fx && !fy {
                    let center = transform.translation.truncate()
                        + velocity.0.normalize_or_zero() * VEHICLE_PREDICT_DISTANCE;

                    let val = fastrand::f32() * range;
                    wander_theta.0 += if fastrand::bool() { val } else { -val };
                    let f = wander_theta.0.sin_cos();

                    let target = center + Vec2::new(f.1, f.0) * VEHICLE_PREDICT_RADIUS;

                    let desired = (target - transform.translation.truncate()).normalize_or_zero()
                        * VEHICLE_WANDER_SPEED;

                    acceleration.apply_force(
                        (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                            * VEHICLE_WANDER_FACTOR,
                        mass,
                    );
                    return;
                }

                let desired = Vec2::new(
                    if fx {
                        if transform.translation.x < WALL_MARGIN {
                            VEHICLE_MAX_SPEED
                        } else {
                            -VEHICLE_MAX_SPEED
                        }
                    } else {
                        velocity.x
                    },
                    if fy {
                        if transform.translation.y < WALL_MARGIN {
                            VEHICLE_MAX_SPEED
                        } else {
                            -VEHICLE_MAX_SPEED
                        }
                    } else {
                        velocity.y
                    },
                );

                acceleration.apply_force(
                    (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                        * VEHICLE_LIMIT_FACTOR,
                    mass,
                );
            },
        );
    }

    // Limit bounds
}

fn update(
    mut vehicle_query: Query<(&mut Velocity, &mut Acceleration, &mut Transform), With<Vehicle>>,
    time: Res<Time>,
    task_pool: Res<TaskPool>,
) {
    vehicle_query.par_for_each_mut(
        &task_pool,
        500,
        |(mut velocity, mut acceleration, mut transform)| {
            velocity.0 =
                (velocity.0 + acceleration.0).clamp(-VEHICLE_MAX_SPEED_VEC, VEHICLE_MAX_SPEED_VEC);

            transform.translation.x += velocity.x * time.delta_seconds();
            transform.translation.y += velocity.y * time.delta_seconds();

            transform.rotation = Quat::from_rotation_z(velocity.y.atan2(velocity.x) - PI / 2.0);

            acceleration.0 *= 0.0;
        },
    );
}

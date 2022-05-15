use std::{f32::consts::PI};

use bevy::{math::const_vec2, prelude::*};
use bevy_prototype_lyon::prelude::*;

use rand::prelude::*;

use crate::{world::WALL_MARGIN, MainCamera, target::TARGET_RADIUS};

const VEHICLE_COUNT: usize = 200;

const VEHICLE_SIZE: f32 = 2.0;
const VEHICLE_MAX_SPEED: f32 = 300.0;
const VEHICLE_MAX_SPEED_VEC: Vec2 = const_vec2!([VEHICLE_MAX_SPEED; 2]);
const VEHICLE_MAX_FORCE: Vec2 = const_vec2!([120.0; 2]);
const VEHICLE_MASS: f32 = 10.0;
const VEHICLE_BODY_COLOR: Color = Color::WHITE;
const VEHICLE_EDGE_COLOR: Color = Color::PINK;
const VEHICLE_WANDER_SPEED: f32 = 150.0;
const VEHICLE_PREDICT_DISTANCE: f32 = VEHICLE_SIZE * 16.0;
const VEHICLE_PREDICT_RADIUS: f32 = VEHICLE_SIZE * 10.0;
const VEHICLE_VIEW_ANGLE: f32 = PI / 8.0;

const VEHICLE_SECONDARY_BODY_COLOR: Color = Color::WHITE;
const VEHICLE_SECONDARY_EDGE_COLOR: Color = Color::ORANGE_RED;

// Distances
const VEHICLE_SEPERATION_DIST: f32 = VEHICLE_SIZE * 6.0;
const VEHICLE_ALIGN_DIST: f32 = VEHICLE_SIZE * 15.0;
const VEHICLE_COHESION_DIST: f32 = VEHICLE_SIZE * 12.0;
const VEHICLE_VIEW_DIST: f32 = VEHICLE_SIZE * 8.0;
const VEHICLE_SEPERATION_DIST_SQ: f32 = VEHICLE_SEPERATION_DIST * VEHICLE_SEPERATION_DIST;
const VEHICLE_ALIGN_DIST_SQ: f32 = VEHICLE_ALIGN_DIST * VEHICLE_ALIGN_DIST;
const VEHICLE_COHESION_DIST_SQ: f32 = VEHICLE_COHESION_DIST * VEHICLE_COHESION_DIST;
const VEHICLE_VIEW_DIST_SQ: f32 = VEHICLE_VIEW_DIST * VEHICLE_VIEW_DIST;

// Force factors
const VEHICLE_SEPERATION_FACTOR: f32 = 2.0;
const VEHICLE_ALIGN_FACTOR: f32 = 1.0;
const VEHICLE_COHESION_FACTOR: f32 = 1.2;
const VEHICLE_LIMIT_FACTOR: f32 = 1.6;
const VEHICLE_TARGET_FACTOR: f32 = 0.7;
const VEHICLE_WANDER_FACTOR: f32 = 0.8;
const VEHICLE_VIEW_FACTOR: f32 = 1.4;

const LINE_WIDTH: f32 = 2.0;

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
            .add_system(vehicle_spawner)
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

fn vehicle_spawner(mut commands: Commands, windows: Res<Windows>, kbd: Res<Input<KeyCode>>,camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>){
    let window = windows.get_primary().unwrap();
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    let (camera, camera_transform) = camera_query.single();

    if let Some(_position) = window.cursor_position(){
        if kbd.pressed(KeyCode::Space){
            let ndc = (_position / window_size) * 2.0 - Vec2::ONE;
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
            let world_pos: Vec2 = world_pos.truncate();

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
                                color: VEHICLE_SECONDARY_BODY_COLOR,
                            }
                        },
                        outline_mode: StrokeMode::new(VEHICLE_SECONDARY_EDGE_COLOR, LINE_WIDTH),
                    },
                    Transform {
                        translation: world_pos.extend(900.0),
                        ..Default::default()
                    },
                ))
                .insert(Vehicle)
                .insert(Velocity(Vec2::ZERO))
                .insert(Acceleration(Vec2::ZERO))
                .insert(Mass(VEHICLE_MASS))
                .insert(WanderTheta(0.0));
        }
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
) {
    // Seperate
    let mut seperate_sum = Vec2::new(0.0, 0.0);
    let mut seperate_count = 0;

    // Align
    let mut align_sum = Vec2::new(0.0, 0.0);
    let mut align_count = 0;

    // Cohesion
    let mut cohesion_sum = Vec2::new(0.0, 0.0);
    let mut cohesion_count = 0;

    // View
    let mut view_sum = Vec2::new(0.0, 0.0);
    let mut view_count = 0;

    other_vehicle_query.for_each(|(other_transform, other_velocity)| {
        let dist = transform
            .translation
            .truncate()
            .distance_squared(other_transform.translation.truncate());

        // Seperate
        if dist > 0.0 {
            if dist <= VEHICLE_SEPERATION_DIST_SQ {
                seperate_sum += (transform.translation.truncate()
                    - other_transform.translation.truncate())
                .normalize_or_zero()
                    / dist.sqrt();

                seperate_count += 1;
            }

            // Align
            if dist <= VEHICLE_ALIGN_DIST_SQ {
                align_sum += other_velocity.0;
                align_count += 1;
            }

            // Cohesion
            if dist <= VEHICLE_COHESION_DIST_SQ {
                cohesion_sum += other_transform.translation.truncate();
                cohesion_count += 1;
            }

            // View
            if dist <= VEHICLE_VIEW_DIST_SQ && view_count == 0 {
                let path =
                    other_transform.translation.truncate() - transform.translation.truncate();

                let angle = velocity.0.angle_between(path);

                if angle.abs() < VEHICLE_VIEW_ANGLE {
                    let val = path.perp();

                    let angle_1 = velocity.0.angle_between(val);
                    let angle_2 = velocity.0.angle_between(-val);

                    if angle_1.abs() < angle_2.abs() {
                        view_sum += val.normalize_or_zero();
                    } else {
                        view_sum -= val.normalize_or_zero();
                    }

                    view_count += 1;
                }
            }
        }
    });

    // Seperate
    if seperate_count > 0 {
        seperate_sum /= seperate_count as f32;
        seperate_sum = seperate_sum.normalize_or_zero() * VEHICLE_MAX_SPEED;

        acceleration.apply_force(
            (seperate_sum - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_SEPERATION_FACTOR,
            mass,
        );
    }

    // Align
    if align_count > 0 {
        align_sum /= align_count as f32;
        align_sum = align_sum.normalize_or_zero() * VEHICLE_MAX_SPEED;

        acceleration.apply_force(
            (align_sum - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_ALIGN_FACTOR,
            mass,
        );
    }

    // Cohesion
    if cohesion_count > 0 {
        let mut desired = Vec2::ZERO;
        cohesion_sum /= cohesion_count as f32;

        seek_steer(&cohesion_sum, &transform, &mut desired);

        acceleration.apply_force(
            (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_COHESION_FACTOR,
            mass,
        );
    }

    // View
    if view_count > 0 {
        view_sum /= view_count as f32;
        view_sum = view_sum.normalize_or_zero() * VEHICLE_MAX_SPEED;

        acceleration.apply_force(
            (view_sum - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE)
                * VEHICLE_VIEW_FACTOR,
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
            vehicle_query.for_each_mut(|(velocity, transform, mut acceleration, mass, _)| {
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
                );
            });

            wander = false;
        }
    }

    let bounds = window_size / 2.0 - WALL_MARGIN;

    // Wander
    if wander {
        let mut rng = rand::thread_rng();
        let range = PI / 8.0;

        vehicle_query.for_each_mut(
            |(velocity, transform, mut acceleration, mass, mut wander_theta)| {
                let fx = transform.translation.x < -bounds.x || transform.translation.x > bounds.x;
                let fy = transform.translation.y < -bounds.y || transform.translation.y > bounds.y;

                flock(
                    &mut acceleration,
                    &transform,
                    &velocity,
                    mass,
                    &other_vehicle_query,
                );

                if !fx && !fy {
                    let center = transform.translation.truncate()
                        + velocity.0.normalize_or_zero() * VEHICLE_PREDICT_DISTANCE;

                    wander_theta.0 += rng.gen_range(-range..=range);

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
}

fn update(
    mut vehicle_query: Query<(&mut Velocity, &mut Acceleration, &mut Transform), With<Vehicle>>,
    time: Res<Time>,
) {
    vehicle_query.for_each_mut(|(mut velocity, mut acceleration, mut transform)| {
        velocity.0 =
            (velocity.0 + acceleration.0).clamp(-VEHICLE_MAX_SPEED_VEC, VEHICLE_MAX_SPEED_VEC);

        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();

        transform.rotation = Quat::from_rotation_z(velocity.y.atan2(velocity.x) - PI / 2.0);

        acceleration.0 *= 0.0;
    });
}

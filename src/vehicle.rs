use std::f32::consts::PI;

use bevy::{math::const_vec2, prelude::*};
use bevy_prototype_lyon::prelude::*;

use rand::prelude::*;

use crate::{world::WALL_MARGIN, MainCamera};

const WORLD_SIZE: Size<f32> = Size {
    height: 300.0,
    width: 300.0,
};

const VEHICLE_SIZE: f32 = 10.0;
const VEHICLE_MAX_SPEED: f32 = 300.0;
const VEHICLE_MAX_SPEED_VEC: Vec2 = const_vec2!([VEHICLE_MAX_SPEED; 2]);
const VEHICLE_MAX_FORCE: Vec2 = const_vec2!([50.0; 2]);
const VEHICLE_MASS: f32 = 10.0;
const VEHICLE_BODY_COLOR: Color = Color::WHITE;
const VEHICLE_EDGE_COLOR: Color = Color::BLUE;
const VEHICLE_WANDER_SPEED: f32 = 200.0;
const VEHICLE_PREDICT_DISTANCE: f32 = 300.0;
const VEHICLE_PREDICT_RADIUS: f32 = 100.0;

const LINE_WIDTH: f32 = 2.0;
pub const TARGET_RADIUS: f32 = 50.0;

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

fn spawn_vehicles(mut commands: Commands) {
    let mut rng = rand::thread_rng();

    for i in 0..500 {
        let x = rng.gen_range(-WORLD_SIZE.width..WORLD_SIZE.width);
        let y = rng.gen_range(-WORLD_SIZE.width..WORLD_SIZE.width);

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

fn calc_movement(
    mut vehicle_query: Query<
        (
            &mut Velocity,
            &Transform,
            &mut Acceleration,
            &Mass,
            &mut WanderTheta,
        ),
        With<Vehicle>,
    >,
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
                let mut desired = world_pos - transform.translation.truncate();

                let dist = desired.length();

                desired = desired.normalize_or_zero();

                if dist < TARGET_RADIUS {
                    desired *= dist / TARGET_RADIUS * VEHICLE_MAX_SPEED;
                } else {
                    desired *= VEHICLE_MAX_SPEED;
                }

                acceleration.apply_force(
                    (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE),
                    mass,
                );
            });

            wander = false;
        }
    }

    // let bounds = window_size / 2.0 - WALL_MARGIN;

    // // Wander
    // if wander {
    //     let mut rng = rand::thread_rng();
    //     let range = PI / 8.0;

    //     vehicle_query.for_each_mut(
    //         |(velocity, transform, mut acceleration, mass, mut wander_theta)| {
    //             let fx = transform.translation.x < -bounds.x || transform.translation.x > bounds.x;
    //             let fy = transform.translation.y < -bounds.y || transform.translation.y > bounds.y;

    //             if !fx && !fy {
    //                 let center = transform.translation.truncate()
    //                     + velocity.0.normalize_or_zero() * VEHICLE_PREDICT_DISTANCE;

    //                 wander_theta.0 += rng.gen_range(-range..=range);

    //                 let f = wander_theta.0.sin_cos();

    //                 let target = center + Vec2::new(f.1, f.0) * VEHICLE_PREDICT_RADIUS;

    //                 let desired = (target - transform.translation.truncate()).normalize_or_zero()
    //                     * VEHICLE_WANDER_SPEED;

    //                 acceleration.apply_force(desired - velocity.0, mass);
    //                 // return;
    //             }

    //             let desired = Vec2::new(
    //                 if fx {
    //                     if transform.translation.x < WALL_MARGIN {
    //                         VEHICLE_MAX_SPEED
    //                     } else {
    //                         -VEHICLE_MAX_SPEED
    //                     }
    //                 } else {
    //                     velocity.x
    //                 },
    //                 if fy {
    //                     if transform.translation.y < WALL_MARGIN {
    //                         VEHICLE_MAX_SPEED
    //                     } else {
    //                         -VEHICLE_MAX_SPEED
    //                     }
    //                 } else {
    //                     velocity.y
    //                 },
    //             );

    //             acceleration.apply_force(
    //                 (desired - velocity.0).clamp(-VEHICLE_MAX_FORCE, VEHICLE_MAX_FORCE),
    //                 mass,
    //             );
    //         },
    //     );
    // }

    // Limit bounds
    // vehicle_query.for_each_mut(|(velocity, transform, mut acceleration, mass, _)| {});
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

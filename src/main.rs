// REFERENCE https://natureofcode.com/book/chapter-6-autonomous-agents/
mod debug;
mod target;
mod vehicle;
mod world;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_prototype_lyon::prelude::*;
use debug::DebugPlugin;
use target::TargetPlugin;
use vehicle::VehiclePlugin;
use world::WorldPlugin;

pub const CLEAR: Color = Color::rgb(0.3, 0.3, 0.3);

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "3D Particle simulation".to_string(),
                width: 1280.,
                height: 720.,
                canvas: Some("#bevy".to_owned()),
                fit_canvas_to_parent: true,
                ..default()
            },
            ..default()
        }))
        .add_startup_system(spawn_camera)
        .add_startup_system(hide_cursor)
        .add_plugin(DebugPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(VehiclePlugin)
        .add_plugin(TargetPlugin)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                top: 1.0,
                bottom: -1.0,
                right: 1.0,
                left: -1.0,
                scaling_mode: ScalingMode::WindowSize,
                ..OrthographicProjection::default()
            },
            ..Default::default()
        })
        .insert(MainCamera);
}

fn hide_cursor(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(false);
}

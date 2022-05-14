// REFERENCE https://natureofcode.com/book/chapter-6-autonomous-agents/
mod debug;
mod target;
mod vehicle;
mod world;

use bevy::{prelude::*, render::camera::ScalingMode, tasks::TaskPool};
use bevy_prototype_lyon::prelude::*;
use debug::DebugPlugin;
use target::TargetPlugin;
use vehicle::VehiclePlugin;
use world::WorldPlugin;

pub const CLEAR: Color = Color::rgb(0.2, 0.2, 0.2);
pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    let height = 900.0;
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            width: height * RESOLUTION,
            height: height,
            title: "Autonomous-characters".to_string(),
            ..Default::default()
        })
        .insert_resource(TaskPool::new())
        .add_startup_system(spawn_camera)
        .add_startup_system(hide_cursor)
        .add_plugins(DefaultPlugins)
        // .add_plugin(DebugPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(ShapePlugin)
        .add_plugin(VehiclePlugin)
        .add_plugin(TargetPlugin)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;

    camera.orthographic_projection.right = 1.0 * RESOLUTION;
    camera.orthographic_projection.left = -1.0 * RESOLUTION;

    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;

    commands.spawn_bundle(camera).insert(MainCamera);
}

fn hide_cursor(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_visibility(false);
}

// REFERENCE https://natureofcode.com/book/chapter-6-autonomous-agents/
mod character;

use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_prototype_lyon::prelude::*;
use character::Character;

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
        .add_startup_system(spawn_camera)
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(Character)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.top = 1.0;
    camera.orthographic_projection.bottom = -1.0;

    camera.orthographic_projection.right = 1.0 * RESOLUTION;
    camera.orthographic_projection.left = -1.0 * RESOLUTION;

    camera.orthographic_projection.scaling_mode = ScalingMode::WindowSize;

    commands.spawn_bundle(camera);
}

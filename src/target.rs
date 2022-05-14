use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

use crate::{vehicle::TARGET_RADIUS, MainCamera};

pub struct TargetPlugin;

#[derive(Component)]
struct Target;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(crate_target).add_system(update);
    }
}

fn crate_target(mut commands: Commands) {
    let shape = shapes::Circle {
        radius: TARGET_RADIUS,
        ..shapes::Circle::default()
    };

    commands
        .spawn_bundle(GeometryBuilder::build_as(
            &shape,
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::NONE),
                outline_mode: StrokeMode::new(Color::RED, 10.0),
            },
            Transform {
                translation: Vec3::new(0.0, 0.0, 10.0),
                ..Default::default()
            },
        ))
        .insert(Target);
}

fn update(
    mut target_query: Query<(&mut Transform, &mut DrawMode), With<Target>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Res<Windows>,
    time: Res<Time>,
) {
    let (mut transform, mut draw_mode) = target_query.single_mut();
    let hue = (time.seconds_since_startup() * 50.0) % 360.0;
    let outline_width = 2.0 + time.seconds_since_startup().sin().abs() * 10.0;

    let window = windows.get_primary().unwrap();
    let (camera, camera_transform) = camera_query.single();

    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

    if let Some(_position) = window.cursor_position() {
        let ndc = (_position / window_size) * 2.0 - Vec2::ONE;
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
        let world_pos: Vec2 = world_pos.truncate();

        transform.translation.x = world_pos.x;
        transform.translation.y = world_pos.y;
    }

    if let DrawMode::Outlined {
        ref mut fill_mode,
        ref mut outline_mode,
    } = *draw_mode
    {
        fill_mode.color = Color::hsl(hue as f32, 1.0, 0.5);
        outline_mode.options.line_width = outline_width as f32;
    }
}

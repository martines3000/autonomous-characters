use bevy::{prelude::*, window::WindowResized};
use bevy_prototype_lyon::prelude::*;

pub const WALL_MARGIN: f32 = 100.0;
const WALL_LINE_WIDTH: f32 = 10.0;
const WALL_COLOR: Color = Color::RED;

pub struct WorldPlugin;

#[derive(Component)]
struct World;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(create_wall)
            .add_system(window_resized);
    }
}

fn create_wall(mut commands: Commands, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();

    let window_size = Vec2::new(window.width() as f32, window.height() as f32);

    let rect = shapes::Rectangle {
        extents: window_size,
        ..shapes::Rectangle::default()
    };

    let builder = GeometryBuilder::new().add(&rect);

    commands
        .spawn_bundle(builder.build(
            DrawMode::Outlined {
                fill_mode: {
                    bevy_prototype_lyon::draw::FillMode {
                        options: FillOptions::non_zero(),
                        color: Color::NONE,
                    }
                },
                outline_mode: StrokeMode::new(WALL_COLOR, WALL_LINE_WIDTH),
            },
            Transform {
                translation: Vec3::new(0.0, 0.0, 900.0),
                ..Default::default()
            },
        ))
        .insert(World)
        .insert(Name::new("Wall"));
}

fn window_resized(
    mut world_query: Query<&mut Path, With<World>>,
    mut events: EventReader<WindowResized>,
) {
    let mut path = world_query.single_mut();

    for event in events.iter() {
        let window_size = Vec2::new(event.width - WALL_MARGIN, event.height - WALL_MARGIN);
        let rect = shapes::Rectangle {
            extents: window_size,
            ..shapes::Rectangle::default()
        };

        *path = ShapePath::build_as(&rect);
    }
}

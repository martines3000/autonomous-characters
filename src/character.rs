use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

const WORLD_SIZE: Size<f32> = Size {
    height: 300.0,
    width: 300.0,
};

const CHARACTER_SIZE: f32 = 10.0;
const LINE_WIDTH: f32 = 2.0;

#[derive(Component)]
pub struct Character;

impl Plugin for Character {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_chars);
    }
}

fn spawn_chars(mut commands: Commands) {
    let mut builder = GeometryBuilder::new();

    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        builder = builder.add(&shapes::RegularPolygon {
            sides: 3,
            center: Vec2::new(
                rng.gen_range(-WORLD_SIZE.width..WORLD_SIZE.width),
                rng.gen_range(-WORLD_SIZE.height..WORLD_SIZE.height),
            ),
            feature: shapes::RegularPolygonFeature::Radius(CHARACTER_SIZE),
            ..shapes::RegularPolygon::default()
        });
    }

    commands.spawn_bundle(builder.build(
        DrawMode::Outlined {
            fill_mode: {
                FillMode {
                    options: FillOptions::non_zero(),
                    color: Color::CYAN,
                }
            },
            outline_mode: StrokeMode::new(Color::BLACK, LINE_WIDTH),
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 900.0),
            ..Default::default()
        },
    ));
}

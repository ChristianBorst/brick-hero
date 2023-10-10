use crate::breaker::{ball_ricochet, Ball, Collider, CollisionEvent, PlayerMessage, Velocity};
use bevy::{prelude::*, sprite::collide_aabb::collide};

pub const WALL_THICKNESS: f32 = 10.0;
pub const LEFT_WALL: f32 = -450.0;
pub const RIGHT_WALL: f32 = 450.0;
pub const BOTTOM_WALL: f32 = -300.;
pub const TOP_WALL: f32 = 300.;

const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

#[derive(Bundle)]
pub struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
    marker: Wall,
}

// Query for walls with this Component
#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct BottomWall;

pub enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}
impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }
    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        assert!(arena_height > 0.);
        let arena_width = RIGHT_WALL - LEFT_WALL;
        assert!(arena_width > 0.);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS)
            }
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.),
                    // Sprite scales must ALWAYS have a z value of to avoid ordering issues
                    scale: location.size().extend(1.),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
            marker: Wall,
        }
    }
}

pub fn setup(commands: &mut Commands) {
    commands.spawn((WallBundle::new(WallLocation::Left), Name::new("WallLeft")));
    commands.spawn((WallBundle::new(WallLocation::Right), Name::new("WallRight")));
    commands.spawn((WallBundle::new(WallLocation::Top), Name::new("WallTop")));
    commands.spawn((
        WallBundle::new(WallLocation::Bottom),
        BottomWall,
        Name::new("WallBottom"),
    ));
}

pub fn check_bottom_wall_collision(
    mut ball_q: Query<(&mut Velocity, &Transform), With<Ball>>,
    mut collider_q: Query<&Transform, (With<BottomWall>, With<Collider>)>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut player_events: EventWriter<PlayerMessage>,
) {
    let (mut ball_v, ball_t) = ball_q.single_mut();
    let ball_size = ball_t.scale.truncate();

    for tform in collider_q.iter_mut() {
        let collision = collide(
            ball_t.translation,
            ball_size,
            tform.translation,
            tform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collision_events.send_default();
            player_events.send(PlayerMessage::JustLostHealth);
            ball_ricochet(collision, &mut ball_v);
        }
    }
}

use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    sprite::MaterialMesh2dBundle,
};

const COLLISION_SOUND_PATH: &str = "sounds/breakout_collision.ogg";

const PADDLE_DIST_FROM_BOTTOM_WALL: f32 = 60.0;
const PADDLE_SIZE: Vec3 = Vec3::new(120., 20., 0.);
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_PADDING: f32 = 10.0;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(-150., -50., 1.);
const BALL_SIZE: Vec3 = Vec3::new(30., 30., 0.);
const BALL_SPEED: f32 = 400.;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const WALL_THICKNESS: f32 = 10.0;
const LEFT_WALL: f32 = -450.0;
const RIGHT_WALL: f32 = 450.0;
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
const BRICK_MARGIN: f32 = 5.;
const BRICK_DIST_FROM_SIDE_WALL: f32 = 60.0;
const BRICK_DIST_FROM_CEILING: f32 = 60.0;
const BRICK_DIST_FROM_PADDLE: f32 = 270.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.);

const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1., 0.5, 0.5);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        // Add frame-based updates
        .add_systems(Update, update_scoreboard)
        // Add 60hz physics update cycle
        .insert_resource(FixedTime::new_from_secs(1.0 / 60.0))
        .add_systems(
            FixedUpdate,
            (
                apply_velocity.before(check_for_collisions),
                move_paddle
                    .before(check_for_collisions)
                    .after(apply_velocity),
                check_for_collisions,
                play_collision_sound.after(check_for_collisions),
            ),
        )
        .run();
}

// The paddle could be a resource, but making a component allows multiple
#[derive(Component)]
struct Paddle;

// Likewise we're likely to have multiple balls (lol)
#[derive(Component)]
struct Ball;

// The paddle and ball will have a velocity, must be a component
// Deref and DerefMut make accessing the contained Vec2 convenient
#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

// Everything but the score needs a collider
#[derive(Component)]
struct Collider;

// Events are added to an EventWriter, read multiple places by EventReaders
// each system that reads events tracks its processed events independently
#[derive(Event, Default)]
struct CollisionEvent;

// There will be many bricks, deployed at level start and
#[derive(Component)]
struct Brick;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource)]
struct Scoreboard {
    score: usize,
}

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

#[derive(Component)]
struct BottomWall;

enum WallLocation {
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
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Create a default camera + all of its systems
    commands.spawn(Camera2dBundle::default());

    let ball_collision_sound = asset_server.load(COLLISION_SOUND_PATH);
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Create the paddle
    let paddle_y = BOTTOM_WALL + PADDLE_DIST_FROM_BOTTOM_WALL;
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., paddle_y, 0.),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        },
        Paddle,
        Collider,
    ));

    // Create the ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::default().into()).into(),
            material: mats.add(ColorMaterial::from(BALL_COLOR)),
            transform: Transform::from_translation(BALL_STARTING_POSITION).with_scale(BALL_SIZE),
            ..default()
        },
        Ball,
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // Create scoreboard
    commands.spawn(
        TextBundle::from_sections([
            // Labels the score
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            // The score value
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: SCORE_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    );

    // Create walls
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn((WallBundle::new(WallLocation::Bottom), BottomWall));

    spawn_bricks(&mut commands)
}

fn spawn_bricks(commands: &mut Commands) {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(BRICK_SIZE.x > 0.);
        assert!(BRICK_SIZE.y > 0.);
    }
    let bricks_width = (RIGHT_WALL - LEFT_WALL) - 2. * BRICK_DIST_FROM_SIDE_WALL;
    let bottom_edge = BOTTOM_WALL + PADDLE_DIST_FROM_BOTTOM_WALL + BRICK_DIST_FROM_PADDLE;
    let bricks_height = TOP_WALL - bottom_edge - BRICK_DIST_FROM_CEILING;
    assert!(bricks_width > BRICK_SIZE.x);
    assert!(bricks_height > BRICK_SIZE.y);

    let brick_cols = (bricks_width / (BRICK_SIZE.x + BRICK_MARGIN)).floor() as usize;
    let brick_rows = (bricks_height / (BRICK_SIZE.y + BRICK_MARGIN)).floor() as usize;

    // Determine the starting position from top left to bottom right, centering the bricks
    let center = LEFT_WALL + (RIGHT_WALL - LEFT_WALL) / 2.0;
    let left_edge = center
        - ((brick_cols as f32) / 2.0 * BRICK_SIZE.x)
        - ((brick_cols - 1) as f32 / 2.0 * BRICK_MARGIN);
    let offset_x = left_edge + BRICK_SIZE.x / 2.0;
    let offset_y = TOP_WALL - BRICK_DIST_FROM_CEILING + BRICK_SIZE.y / 2.0;

    for row in 0..brick_rows {
        for col in 0..brick_cols {
            let brick_pos = Vec2::new(
                offset_x + col as f32 * (BRICK_SIZE.x + BRICK_MARGIN),
                offset_y - row as f32 * (BRICK_SIZE.y + BRICK_MARGIN),
            );

            commands.spawn((brick_sprite(brick_pos), Brick, Collider));
        }
    }
}

fn brick_sprite(position: Vec2) -> SpriteBundle {
    SpriteBundle {
        sprite: Sprite {
            color: BRICK_COLOR,
            ..default()
        },
        transform: Transform {
            translation: position.extend(0.),
            scale: BRICK_SIZE.extend(1.0),
            ..default()
        },
        ..default()
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut text_q: Query<&mut Text>) {
    // text_q holds the setup values put in the TextBundle
    let mut text = text_q.single_mut();
    // Update the empty section given the SCORE_COLOR
    text.sections[1].value = scoreboard.score.to_string();
}

// Handle paddle movement via A/D or Left/Right arrows, keeping the paddle within the play area
// via these bounds
const PADDLE_LEFT_BOUND: f32 =
    LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
const PADDLE_RIGHT_BOUND: f32 =
    RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
fn move_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut paddle_q: Query<&mut Transform, With<Paddle>>,
    time_step: Res<FixedTime>,
) {
    let mut paddle_tform = paddle_q.single_mut();
    let mut dir = 0.0;

    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    if left {
        dir -= 1.0;
    }
    if right {
        dir += 1.0;
    }

    let new_pos = paddle_tform.translation.x + dir * PADDLE_SPEED * time_step.period.as_secs_f32();

    paddle_tform.translation.x = new_pos.clamp(PADDLE_LEFT_BOUND, PADDLE_RIGHT_BOUND)
}

fn apply_velocity(mut tform_vel_q: Query<(&mut Transform, &Velocity)>, time_step: Res<FixedTime>) {
    for (mut tform, velocity) in &mut tform_vel_q {
        tform.translation.x += velocity.x * time_step.period.as_secs_f32();
        tform.translation.y += velocity.y * time_step.period.as_secs_f32();
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_q: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_q: Query<(Entity, &Transform, Option<&Brick>, Option<&BottomWall>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_v, ball_t) = ball_q.single_mut();
    let ball_size = ball_t.scale.truncate();

    for (collider_ent, tform, is_brick, is_bottom) in &collider_q {
        let collision = collide(
            ball_t.translation,
            ball_size,
            tform.translation,
            tform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collision_events.send_default();
            if is_brick.is_some() {
                scoreboard.score += 10;
                commands.entity(collider_ent).despawn();
            }
            if is_bottom.is_some() {
                scoreboard.score -= 100;
            }
            let reflect_x: bool;
            let reflect_y: bool;
            match collision {
                Collision::Left => {
                    reflect_x = ball_v.x > 0.;
                    reflect_y = false
                }
                Collision::Right => {
                    reflect_x = ball_v.x < 0.;
                    reflect_y = false
                }
                Collision::Top => {
                    reflect_y = ball_v.y < 0.;
                    reflect_x = false
                }
                Collision::Bottom => {
                    reflect_y = ball_v.y > 0.;
                    reflect_x = false
                }
                Collision::Inside => {
                    reflect_x = false;
                    reflect_y = false
                }
            }
            if reflect_x {
                ball_v.x *= -1.;
            }
            if reflect_y {
                ball_v.y *= -1.;
            }
        }
    }
}

fn play_collision_sound(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    if !collision_events.is_empty() {
        collision_events.clear();
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}

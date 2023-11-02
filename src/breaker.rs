use core::f32::consts::PI;

use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::{
        collide_aabb::{collide, Collision},
        MaterialMesh2dBundle,
    },
};
use lerp::Lerp;

use crate::{
    app_state::{AppState, AppStateTransition},
    bricks::{spawn_bricks, Brick, BRICK_COLORS},
    health::{Health, HealthDisplay, HealthDisplayBundle},
    misc::blink::{blink, Blinking},
    scoreboard::{update_scoreboard, Scoreboard, ScoreboardBundle},
    walls::{self, Wall},
};

#[derive(Resource, Deref, DerefMut, PartialEq, Eq)]
pub struct CurrentState(pub GameState);

#[derive(PartialEq, Eq, Clone, States, Debug, Hash)]
pub enum GameState {
    Uninitialized,
    Playing,
    Paused,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Uninitialized
    }
}

// Events related to starting the game, creating levels, playing, pausing
#[derive(Event, Clone, Debug)]
pub enum GameStateTransition {
    ToUninitialized,
    ToPlayGame,
    ToHaltGame,
    NextLevel,
    ToGameOver,
    // TODO:
    // RestartGame,
}

// Events related to player health, death
#[derive(Event, Clone)]
pub enum PlayerMessage {
    JustLostHealth,
}

// The current number of bricks in the level
#[derive(Resource, Deref, DerefMut)]
pub struct BrickTracker(usize);

// The current level
#[derive(Resource, Deref, DerefMut)]
pub struct Level(usize);

// The paddle could be a resource, but making a component allows multiple
#[derive(Component)]
pub struct Paddle;

// The paddle's movement can influence where the ball goes
#[derive(Resource, Deref, DerefMut)]
pub struct PaddleMomentum(f32);

#[derive(Resource, Deref, DerefMut)]
pub enum ControlStyle {
    Edges,
    Momentum,
    Unaltered,
}

// Likewise we're likely to have multiple balls (lol)
#[derive(Component)]
pub struct Ball;

// The paddle and ball will have a velocity, must be a component
// Deref and DerefMut make accessing the contained Vec2 convenient
#[derive(Component, Deref, DerefMut)]
pub struct Velocity(Vec2);

// Everything but the score needs a collider
#[derive(Component)]
pub struct Collider;

// Events are added to an EventWriter, read multiple places by EventReaders
// each system that reads events tracks its processed events independently
#[derive(Event, Default)]
pub struct CollisionEvent;

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

const COLLISION_SOUND_PATH: &str = "sounds/breakout_collision.ogg";

pub const FIXED_TIME_TICKS_PER_SECOND: f32 = 1.0 / 60.0;

pub const PADDLE_DIST_FROM_BOTTOM_WALL: f32 = 60.0;
pub const PADDLE_SIZE: Vec3 = Vec3::new(120., 20., 0.);
pub const PADDLE_MAX_INFLUENCE: f32 = PI / 2.;
const PADDLE_LEFT_BOUND: f32 =
    walls::LEFT_WALL + walls::WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
const PADDLE_RIGHT_BOUND: f32 =
    walls::RIGHT_WALL - walls::WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
const PADDLE_MAX_MOMENTUM: f32 = 7.;
const PADDLE_LERP: f32 = 0.10;
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_PADDING: f32 = 10.0;
const PADDLE_STARTING_POSITION_X: f32 = 0.;
const PADDLE_STARTING_POSITION_Y: f32 = walls::BOTTOM_WALL + PADDLE_DIST_FROM_BOTTOM_WALL;

const BALL_STARTING_POSITION: Vec3 = Vec3::new(-150., -50., 1.);
const BALL_SIZE: Vec3 = Vec3::new(30., 30., 0.);
const BALL_STARTING_SPEED: f32 = 300.;
const BALL_SPEED: f32 = 300.;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

const SCOREBOARD_FONT_SIZE: f32 = 40.;
const SCOREBOARD_TEXT_PADDING: f32 = 5.;
const HEALTH_Y_POS: f32 = 650.;

const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1., 0.5, 0.5);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

const PLAYER_STARTING_HEALTH: usize = 3;

const BLINK_DURATION: f64 = 1.0;

pub struct BreakoutGamePlugin;
impl Plugin for BreakoutGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scoreboard { score: 0 })
            .insert_resource(CurrentState(GameState::Uninitialized))
            .insert_resource(BrickTracker(0))
            .insert_resource(Level(1))
            .insert_resource(Health(PLAYER_STARTING_HEALTH))
            .insert_resource(PaddleMomentum(0.))
            .insert_resource(ControlStyle::Edges)
            .add_event::<CollisionEvent>()
            .add_event::<GameStateTransition>()
            .add_event::<PlayerMessage>()
            // .add_systems(Startup, (setup, walls::setup)) // TODO: Call these manually when AS::InGame && GS::Uninitialized
            // Add frame-based updates that always run while AS::InGame
            .add_systems(
                Update,
                (
                    // Run these regardless of if the game is currently playing
                    transition_game,
                    manage_game.after(transition_game),
                    game_aux_keys_handler.after(manage_game),
                    // Run these only if the game is currently playing
                )
                    .run_if(state_exists_and_equals(AppState::InGame)),
            )
            // Add frame-based updates that only run while GS::Playing
            .add_systems(
                Update,
                (
                    health_handler,
                    update_scoreboard,
                    blink,
                    play_collision_sound,
                )
                    .run_if(resource_equals(CurrentState(GameState::Playing))),
            )
            // Add 60hz physics update cycle
            .insert_resource(FixedTime::new_from_secs(FIXED_TIME_TICKS_PER_SECOND))
            .add_systems(
                FixedUpdate,
                // Only run these if the game is playing
                (
                    // apply_velocity,
                    move_ball,
                    update_paddle_momentum.before(update_paddle),
                    update_paddle,
                    check_brick_collisions.after(apply_velocity),
                    walls::check_bottom_wall_collision.after(apply_velocity),
                    check_paddle_collision.after(apply_velocity),
                    check_wall_collision.after(apply_velocity),
                )
                    .run_if(resource_equals(CurrentState(GameState::Playing))),
            );
    }
}

fn transition_game(
    mut game_state: ResMut<CurrentState>,
    mut game_transition_reqs: EventReader<GameStateTransition>,
    mut commands: Commands,
    entities_q: Query<Entity>,
    mut ball_q: Query<&mut Transform, (With<Ball>, Without<Paddle>)>,
    mut paddle_q: Query<&mut Transform, (With<Paddle>, Without<Ball>)>,
    mut level: ResMut<Level>,
    mut brick_tracker: ResMut<BrickTracker>,
    asset_server: Res<AssetServer>,
) {
    for transition in game_transition_reqs.iter() {
        info!(
            "Game Transition Request: {:?} -> {:?}",
            **game_state, transition
        );
        match transition {
            GameStateTransition::ToUninitialized => {
                // Delete all entities to restart the game
                for ent in entities_q.iter() {
                    commands.entity(ent).despawn_recursive();
                }
                **game_state = GameState::Uninitialized;
            }
            GameStateTransition::ToPlayGame => **game_state = GameState::Playing,
            GameStateTransition::ToHaltGame => **game_state = GameState::Paused,
            GameStateTransition::NextLevel => {
                // TODO: Detect win, display different UI
                **level += 1; // Advance the level
                              // Spawn the next level's bricks and update te brick tracker
                **brick_tracker = spawn_bricks(&mut commands, **level, &asset_server);

                // Reset the ball and paddle positions
                let mut ball = ball_q.iter_mut().next().unwrap();
                ball.translation = BALL_STARTING_POSITION;
                let mut paddle = paddle_q.iter_mut().next().unwrap();
                paddle.translation =
                    Vec3::new(PADDLE_STARTING_POSITION_X, PADDLE_STARTING_POSITION_Y, 0.);

                // Here would be where we reset score and/or health between levels
            }
            GameStateTransition::ToGameOver => {
                // TODO: Add game over screen
                panic!("You lost");
            }
        }
    }
}

fn manage_game(
    game_state: Res<CurrentState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut game_state_msgs: EventWriter<GameStateTransition>,
    // mut app_state_msgs: EventWriter<AppStateTransition>,
    mut brick_tracker: ResMut<BrickTracker>,
    health: Res<Health>,
    level: Res<Level>,
) {
    match **game_state {
        GameState::Uninitialized => {
            setup(&mut commands, &mut meshes, &mut mats, &asset_server);
            **brick_tracker = spawn_bricks(&mut commands, **level, &asset_server);
            game_state_msgs.send(GameStateTransition::ToPlayGame);
        }
        GameState::Playing => {
            if **brick_tracker == 0 {
                game_state_msgs.send(GameStateTransition::NextLevel)
            }
            if **health == 0 {
                game_state_msgs.send(GameStateTransition::ToGameOver)
            }
        }
        _ => {}
    }
}

fn setup(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    mats: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    info!("Start breaker setup");
    // Create a default camera + all of its systems
    commands.spawn(Camera2dBundle::default());

    let ball_collision_sound = asset_server.load(COLLISION_SOUND_PATH);
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Create the paddle
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(PADDLE_STARTING_POSITION_X, PADDLE_STARTING_POSITION_Y, 0.),
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
        Name::new("Paddle"),
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
        // Ball doesn't get a collider, collisions are detected manually but with other colliders
        Velocity(INITIAL_BALL_DIRECTION.normalize()),
        Name::new("Ball"),
    ));

    // Create scoreboard
    commands.spawn(ScoreboardBundle::new(
        SCOREBOARD_FONT_SIZE,
        TEXT_COLOR,
        SCORE_COLOR,
        "Score: ",
        Vec2::new(SCOREBOARD_TEXT_PADDING, SCOREBOARD_TEXT_PADDING),
        Some("0"),
    ));

    // Create Health tracker
    commands.spawn(HealthDisplayBundle::new(
        SCOREBOARD_FONT_SIZE,
        TEXT_COLOR,
        SCORE_COLOR,
        "Health: ",
        Vec2::new(SCOREBOARD_TEXT_PADDING, HEALTH_Y_POS),
        Some("0"),
    ));
    walls::setup(commands);
}

// Updates the paddle's momentum param based on user input. Applies a force to the left with A/<- and to the right with D/<-
fn update_paddle_momentum(
    mut paddle_momentum: ResMut<PaddleMomentum>,
    keyboard_input: Res<Input<KeyCode>>,
    time_step: Res<FixedTime>,
) {
    let left = keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left);
    let right = keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right);

    let dir = match (left, right) {
        (true, false) => -1.,
        (false, true) => 1.,
        _ => 0.,
    };

    // Change the momentum towards the movement direction, scaled by speed and time
    // PADDLE_LERP basically gives the paddle high mass close to 0 and low mass close to 1
    **paddle_momentum = paddle_momentum
        .lerp(
            dir * PADDLE_SPEED * time_step.period.as_secs_f32(),
            PADDLE_LERP,
        )
        .clamp(-PADDLE_MAX_MOMENTUM, PADDLE_MAX_MOMENTUM);
}

// Moves the paddle based on the current momentum value
fn update_paddle(
    mut paddle_q: Query<&mut Transform, With<Paddle>>,
    paddle_momentum: Res<PaddleMomentum>,
) {
    let mut paddle_tform = paddle_q.single_mut();
    let start = paddle_tform.translation.x;
    let x = start + **paddle_momentum;
    let x = x.clamp(PADDLE_LEFT_BOUND, PADDLE_RIGHT_BOUND);

    let delta = x - start; // Calculate delta off actual movement since paddle is bounded by walls
    paddle_tform.translation.x = x;
}

fn apply_velocity(
    mut tform_vel_q: Query<(&mut Transform, &Velocity), Without<Ball>>,
    time_step: Res<FixedTime>,
) {
    for (mut tform, velocity) in &mut tform_vel_q {
        tform.translation.x += velocity.x * time_step.period.as_secs_f32();
        tform.translation.y += velocity.y * time_step.period.as_secs_f32();
    }
}

// The ball moves differently from other objects with velocity, since we do more manual
// control of where it goes. It has a given velocity which is treated as a unit vector
// and is scaled by the speed and duration of this physics tick
fn move_ball(
    mut ball_tform_vel: Query<(&mut Transform, &Velocity), With<Ball>>,
    time_step: Res<FixedTime>,
) {
    let (mut ball_t, ball_v) = ball_tform_vel.single_mut();
    let movement: Vec2 = ball_v.0 * time_step.period.as_secs_f32() * BALL_SPEED;
    ball_t.translation += movement.extend(0.);
}

// Checks for collsions with bricks
fn check_brick_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_q: Query<(&mut Velocity, &Transform), With<Ball>>,
    mut collider_q: Query<(Entity, &Transform, &mut Brick, &mut Sprite), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
    mut brick_tracker: ResMut<BrickTracker>,
) {
    let (mut ball_v, ball_t) = ball_q.single_mut();
    let ball_size = ball_t.scale.truncate();

    for (collider_ent, tform, mut brick, mut sprite) in collider_q.iter_mut() {
        let collision = collide(
            ball_t.translation,
            ball_size,
            tform.translation,
            tform.scale.truncate(),
        );
        if let Some(collision) = collision {
            collision_events.send(CollisionEvent);
            brick_collision(
                &mut scoreboard,
                &mut commands,
                &mut brick_tracker,
                collider_ent,
                &mut brick,
                &mut sprite,
            );
            ball_ricochet(collision, &mut ball_v);
        }
    }
}

fn check_paddle_collision(
    mut ball_q: Query<(&mut Velocity, &Transform), With<Ball>>,
    mut collider_q: Query<
        &Transform,
        (
            With<Collider>,
            With<Paddle>,
            Without<Brick>,
            Without<walls::BottomWall>,
        ),
    >,
    mut collision_events: EventWriter<CollisionEvent>,
    paddle_momentum: Res<PaddleMomentum>,
    control_style: Res<ControlStyle>,
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
            // ball_ricochet mutates ball_v to be the already reflected vector
            ball_ricochet(collision, &mut ball_v);
            if let Collision::Bottom | Collision::Top = collision {
                match control_style {
                    ControlStyle::Edges => ball_influence_edges(collision, &mut ball_v, &tform),
                    ControlStyle::Momentum => {
                        ball_influence_momentum(collision, &mut ball_v, &paddle_momentum, &tform)
                    }
                    ControlStyle::Unaltered => {}
                }
            }
            break; // Do not collide with multiple paddles in the same frame
        }
    }
}

// Changes ball_v based on the location of the ball's collision with the paddle
// NOT A SYSTEM
fn ball_influence_edges(collision: Collision, ball_vel: &Velocity, paddle_t: Transform) {
    let reflected_angle = ball_v.angle_between(Vec2::Y);

    // TODO: Calculate the influence of the paddle on the ball's reflection angle, then factor in to desired_angle below
    let paddle_influence = PADDLE_MAX_INFLUENCE * (**paddle_momentum / PADDLE_MAX_MOMENTUM);

    // Otherwise, adjust the movement by the offset * influence
    let desired_angle = (reflected_angle + paddle_influence).clamp(-PI / 2., PI / 2.);
    let magnitude = ball_v.0.length(); // Preserve momentum by tracking magnitude
    ball_v.0 = Vec2::new(desired_angle.sin(), reflected_angle.cos()).normalize() * magnitude;
}

// Changes ball_v based on the momentum of the paddle at the time of collision
// NOT A SYSTEM
fn ball_influence_momentum(
    collision: Collision,
    ball_v: &Velocity,
    paddle_momentum: &PaddleMomentum,
    tform: Transform,
) {
    let reflected_angle = ball_v.angle_between(Vec2::Y);
    // Convert the current momentum into a [-1, 1] range by dividing by PADDLE_MAX_SPEED, and scale by max influence to get the desired influence
    let momentum_influence = PADDLE_MAX_INFLUENCE * (**paddle_momentum / PADDLE_MAX_MOMENTUM);

    // Otherwise, adjust the movement by the offset * influence
    let desired_angle = (reflected_angle + momentum_influence).clamp(-PI / 2., PI / 2.);
    let magnitude = ball_v.0.length(); // Preserve momentum by tracking magnitude
    ball_v.0 = Vec2::new(desired_angle.sin(), reflected_angle.cos()).normalize() * magnitude;
}

// fn ball_influence_edges()

fn check_wall_collision(
    mut ball_q: Query<(&mut Velocity, &Transform), With<Ball>>,
    mut collider_q: Query<
        &Transform,
        (
            With<Collider>,
            With<Wall>,
            Without<Paddle>,
            Without<Brick>,
            Without<walls::BottomWall>,
        ),
    >,
    mut collision_events: EventWriter<CollisionEvent>,
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
            ball_ricochet(collision, &mut ball_v);
        }
    }
}

// Adjusts the ball velocity as a result of the collision type
pub fn ball_ricochet(collision: Collision, ball_v: &mut Velocity) {
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

// Updates score + brick strength, despawns bricks, changes brick colors
fn brick_collision(
    scoreboard: &mut ResMut<Scoreboard>,
    commands: &mut Commands,
    brick_tracker: &mut ResMut<BrickTracker>,
    brick_ent: Entity,
    brick: &mut Brick,
    sprite: &mut Sprite,
) {
    scoreboard.score += 10;
    // Decrease brick strength (0 -> despawn)
    **brick -= 1;
    if **brick == 0 {
        commands.entity(brick_ent).despawn_recursive();
        ***brick_tracker -= 1;
        return;
    }
    // At this point **brick > 0, decrement is safe
    sprite.color = BRICK_COLORS[(**brick - 1) as usize];
}

const COLLISION_SOUND_DELAY: f32 = 0.1;
// Plays a sound any time there is >= 1 CollisionEvent message
// WARNING: Does not work in FixedUpdate (idk y) + Requires use of CollisionSound so must run only while Playing
fn play_collision_sound(
    mut delay: Local<f32>,
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
    time: Res<Time>,
) {
    *delay += time.delta_seconds();
    if !collision_events.is_empty() {
        collision_events.clear();
        if *delay >= COLLISION_SOUND_DELAY {
            *delay = 0.;
            commands.spawn(AudioBundle {
                source: sound.0.clone(),
                settings: PlaybackSettings::DESPAWN,
            });
        }
    }
}

// Decrements Health, causes death and loss of health blinking
#[allow(clippy::too_many_arguments)]
fn health_handler(
    mut commands: Commands,
    mut state_msgs: EventWriter<AppStateTransition>,
    mut player_msgs: EventReader<PlayerMessage>,
    mut health: ResMut<Health>,
    mut text_q: Query<&mut Text, With<HealthDisplay>>,
    mut paddle_q: Query<(Entity, Option<&Blinking>), With<Paddle>>,
    mut ball_q: Query<Entity, With<Ball>>,
) {
    for msg in player_msgs.iter() {
        match msg {
            PlayerMessage::JustLostHealth => {
                let (paddle, blinking) = paddle_q.single_mut();
                if blinking.is_some() {
                    continue; // Do not remove health while they are blinking
                }

                if **health == 0 {
                    state_msgs.send(AppStateTransition::ToMainMenu); // TODO: Show Game Over screen
                } else {
                    **health -= 1;
                    // Make the paddle blink
                    commands.entity(paddle).insert(Blinking(Timer::new(
                        Duration::from_secs_f64(BLINK_DURATION),
                        TimerMode::Once,
                    )));
                    // Make the ball blink
                    let ball = ball_q.single_mut();
                    commands.entity(ball).insert(Blinking(Timer::new(
                        Duration::from_secs_f64(BLINK_DURATION),
                        TimerMode::Once,
                    )));
                }
                // text_q holds the setup values put in the TextBundle
                let mut text = text_q.single_mut();
                // Update the empty section given the SCORE_COLOR
                text.sections[1].value = health.0.to_string();
            }
        }
    }
}

// TODO: Refactor into game state transition logic

// Handles presses of keys not directly related to the brick breaker gameplay
// like pause and resume
fn game_aux_keys_handler(
    mut game_msgs: EventWriter<GameStateTransition>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Return) {
        game_msgs.send(GameStateTransition::ToPlayGame);
    }

    if keys.just_pressed(KeyCode::Escape) {
        game_msgs.send(GameStateTransition::ToHaltGame);
    }
}

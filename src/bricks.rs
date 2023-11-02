use bevy::prelude::*;

use crate::breaker::{Collider, PADDLE_DIST_FROM_BOTTOM_WALL};
use crate::walls::{BOTTOM_WALL, LEFT_WALL, RIGHT_WALL, TOP_WALL};

const BRICK_SIZE: Vec2 = Vec2::new(100., 50.);
const BRICK_MARGIN: f32 = 5.;
const BRICK_DIST_FROM_SIDE_WALL: f32 = 60.0;
const BRICK_DIST_FROM_CEILING: f32 = 60.0;
const BRICK_DIST_FROM_PADDLE: f32 = 270.0;
pub const BRICK_COLORS: [Color; 3] = [
    Color::rgb(0.5, 0.5, 1.),
    Color::rgb(1., 0.5, 1.),
    Color::rgb(0.5, 1., 0.5),
];

#[derive(Deref, DerefMut)]
// Describes the organization of bricks in rows, with the given strengths
pub struct BrickLayout([u8; 5]);

const LEVELS: [BrickLayout; 5] = [
    BrickLayout([1, 1, 1, 1, 1]),
    BrickLayout([2, 1, 1, 1, 2]),
    BrickLayout([1, 1, 2, 2, 3]),
    BrickLayout([1, 3, 1, 3, 1]),
    BrickLayout([3, 3, 1, 3, 3]),
];

// There will be many bricks, deployed at level start and
#[derive(Component, Clone, Copy, Deref, DerefMut)]
pub struct Brick(u8);

pub fn spawn_bricks(
    commands: &mut Commands,
    level: usize,
    asset_server: &Res<AssetServer>,
) -> usize {
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

    let brick_cols = (bricks_width / (BRICK_SIZE.x + BRICK_MARGIN)).floor() as u32;
    let brick_rows = LEVELS.len();

    // Determine the starting position from top left to bottom right, centering the bricks
    let center = LEFT_WALL + (RIGHT_WALL - LEFT_WALL) / 2.0;
    let left_edge = center
        - ((brick_cols as f32) / 2.0 * BRICK_SIZE.x)
        - ((brick_cols - 1) as f32 / 2.0 * BRICK_MARGIN);
    let offset_y = TOP_WALL - BRICK_DIST_FROM_CEILING + BRICK_SIZE.y / 2.0;

    let brick_layout = &LEVELS[level];

    let mut num_bricks = 0;
    for row in 0..brick_rows {
        let row_strength = brick_layout[row];
        let row_y = offset_y - row as f32 * (BRICK_SIZE.y + BRICK_MARGIN);
        num_bricks += spawn_brick_row(
            commands,
            row_strength,
            row_y,
            left_edge,
            brick_cols,
            asset_server,
        );
    }
    num_bricks
}

pub fn spawn_brick_row(
    commands: &mut Commands,
    brick_strength: u8,
    y_position: f32,
    left_edge: f32,
    cols: u32,
    asset_server: &Res<AssetServer>,
) -> usize {
    let offset_x = left_edge + BRICK_SIZE.x / 2.0;

    let mut spawned = 0;
    for col in 0..cols {
        let brick_pos = Vec2::new(
            offset_x + col as f32 * (BRICK_SIZE.x + BRICK_MARGIN),
            y_position,
        );
        let brick = Brick(brick_strength);
        commands.spawn((
            brick_sprite(brick_pos, brick.clone(), asset_server),
            brick,
            Collider,
            Name::new(format!("Brick{spawned}")),
        ));
        spawned += 1;
    }
    spawned
}

fn brick_sprite(position: Vec2, brick: Brick, asset_server: &Res<AssetServer>) -> SpriteBundle {
    let color = BRICK_COLORS[(brick.0 - 1) as usize];
    SpriteBundle {
        texture: asset_server.load("images/holo-brick.png"),
        transform: Transform {
            translation: position.extend(0.),
            scale: BRICK_SIZE.extend(1.),
            ..default()
        },
        sprite: Sprite {
            custom_size: Some(Vec2::new(1., 1.)),
            color, // This color gets multiplied with the texture color in the sprite shader
            ..default()
        },
        ..default()
    }
}

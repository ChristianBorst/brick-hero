use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use app_state::AppStatePlugin;
use breaker::BreakoutGamePlugin;
use ui::UIPlugin;

pub mod app_state;
pub mod breaker;
pub mod bricks;
pub mod health;
pub mod misc;
pub mod scoreboard;
pub mod ui;
pub mod walls;

const CLEAR_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        BreakoutGamePlugin,
        UIPlugin,
        AppStatePlugin,
        WorldInspectorPlugin::new(),
    ))
    .insert_resource(ClearColor(CLEAR_COLOR));

    app.run()
}

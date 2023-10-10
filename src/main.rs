use crate::app_state::AppState;
use crate::ui::register_ui;
use app_state::{handle_transition_request, AppStateTransition};
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use breaker::register_breaker;

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
    app.add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(ClearColor(CLEAR_COLOR))
        .add_state::<AppState>()
        .add_event::<AppStateTransition>()
        .add_systems(Update, handle_transition_request);
    register_ui(&mut app);
    register_breaker(&mut app);

    app.run()
}

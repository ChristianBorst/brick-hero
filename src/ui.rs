use bevy::prelude::*;
use bevy_iced::iced::{
    widget::{text, Button, Row},
    Alignment,
};
use bevy_iced::{IcedContext, IcedPlugin};

use crate::app_state::{AppState, AppStateTransition};

pub fn register_ui(app: &mut App) {
    app.add_plugins(IcedPlugin::default()).add_systems(
        Update,
        (menu_sys.run_if(
            state_exists_and_equals(AppState::MainMenu)
                .or_else(state_exists_and_equals(AppState::GameOver)),
        ),),
    );
}

// This is registered to run only if MainMenuToggle has a true value
pub fn menu_sys(mut ctx: IcedContext<AppStateTransition>, state: Res<State<AppState>>) {
    let curr_state = state.get();
    match curr_state {
        AppState::InGame => panic!("menu_sys executed while playing"), // TODO: Implement
        AppState::MainMenu => main_menu(&mut ctx),
        AppState::GameOver => {} // TODO: Implement
    };
}

fn main_menu(ctx: &mut IcedContext<AppStateTransition>) {
    let row = Row::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(Button::new(text("Start Game")).on_press(AppStateTransition::ToInGame));

    ctx.display(row);
}

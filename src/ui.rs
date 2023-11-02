use bevy::prelude::*;
use bevy_iced::iced::{
    alignment::{Horizontal, Vertical},
    widget::{text, Button, Column, Container},
    Alignment, Length,
};
use bevy_iced::{IcedContext, IcedPlugin};

use crate::app_state::{AppState, AppStateTransition};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(IcedPlugin::default()).add_systems(
            Update,
            (menu_sys.run_if(
                state_exists_and_equals(AppState::MainMenu)
                    .or_else(state_exists_and_equals(AppState::GameOver)),
            ),),
        );
    }
}

// This is registered to run only if MainMenuToggle has a true value
pub fn menu_sys(mut ctx: IcedContext<AppStateTransition>, state: Res<State<AppState>>) {
    let curr_state = state.get();
    match curr_state {
        AppState::InGame => panic!("menu_sys executed while playing"),
        AppState::MainMenu => main_menu(&mut ctx),
        _ => {} // TODO: Implement Game over
    };
}

fn main_menu(ctx: &mut IcedContext<AppStateTransition>) {
    // Make all buttons
    let start_button = Button::new(
        text("Start Game")
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    .on_press(AppStateTransition::ToInGame)
    .width(150.)
    .height(50.);
    let quit_button = Button::new(
        text("Exit")
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
    .on_press(AppStateTransition::ToExit)
    .width(150.)
    .height(50.);

    // let handle = image::Handle::from_path("assets/images/test.png");
    // let image = image::viewer(handle);
    // let image_container = Container::new(Container::new(image).children().push(text("text")));

    // Align the buttons in a column
    let column = Column::new()
        .spacing(10)
        .align_items(Alignment::Center)
        .push(start_button)
        // .push(image_container)
        .push(quit_button);

    // Put the column in a container which is on the left side
    let cont = Container::new(column)
        .center_x()
        .width(Length::Fixed(500.))
        .center_y()
        .height(Length::Fill);

    ctx.display(cont);
}

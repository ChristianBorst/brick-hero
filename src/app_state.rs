use bevy::prelude::*;

// Controls the function of the app, from displaying a menu to actually playing the game
// Acts like a resource because it is registered as a State with App
#[derive(Debug, Clone, Eq, PartialEq, Hash, States)]
pub enum AppState {
    MainMenu,
    InGame,
    GameOver,
}

impl Default for AppState {
    fn default() -> Self {
        AppState::MainMenu
    }
}

// Controls state transitions through this Event, write a state transition request
// and the `handle_transition_request()` system will queue it with Bevy
#[derive(Event, Clone, Debug)]
pub enum AppStateTransition {
    ToMainMenu,
    ToInGame,
    ToGameOver,
}

// Queues state transitions, centralizing the location where AppState is modified
pub fn handle_transition_request(
    mut transition_requests: EventReader<AppStateTransition>,
    mut next_state: ResMut<NextState<AppState>>,
    app_state: Res<State<AppState>>,
) {
    for request in transition_requests.iter() {
        info!("Transition Request: {:?} -> {:?}", app_state.get(), request);
        match request {
            AppStateTransition::ToMainMenu => next_state.set(AppState::MainMenu),
            AppStateTransition::ToInGame => next_state.set(AppState::InGame),
            AppStateTransition::ToGameOver => next_state.set(AppState::GameOver),
        }
    }
}

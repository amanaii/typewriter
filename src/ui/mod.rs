pub mod screens;
pub mod theme;

use ratatui::Frame;

use crate::app::AppState;

pub fn render(frame: &mut Frame, state: &AppState) {
    match state {
        AppState::Home(_) => screens::home::render(frame, state),
        AppState::Test(_) => screens::test::render(frame, state),
        AppState::Results(_) => screens::results::render(frame, state),
        AppState::Quitting => {}
    }
}

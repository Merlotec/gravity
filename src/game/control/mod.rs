pub mod camera;

/// The current game state that the player is experiencing.
#[derive(Copy, Clone)]
pub enum ViewState {
    Spatial,
    System,
}

impl Default for ViewState {
    fn default() -> Self {
        ViewState::Spatial
    }
}

#[derive(Default)]
pub struct GameState {
    pub view_state: ViewState,
}
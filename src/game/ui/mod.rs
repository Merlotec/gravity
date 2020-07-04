pub mod hud;
pub mod status;
pub mod crosshair;
pub mod description;
pub mod ability;
pub mod font;
pub mod marker;
pub mod select_character;
pub mod select_all_button;
pub mod turn_notification;
pub mod map_notification;
pub mod hack;
pub mod banner;
pub mod select_rank;
pub mod dialogue;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UiDisengageEvent {
    Cancel,
    TargetChanged,
    NegatingInput,
    AbilitySelected,
}
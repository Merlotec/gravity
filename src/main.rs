#[macro_use]
extern crate amethyst;
#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate specs_physics as physics;
#[macro_use]
extern crate specs_derive;

use amethyst::prelude::*;

use crate::game::combat::CombatData;
use amethyst::{
    LoggerConfig,
    StdoutLog,
    LogLevelFilter,
};
use crate::state::map_state::MapState;

pub mod core;
pub mod state;
pub mod game;

pub mod client;
pub mod server;

pub const STD_FOV: f32 = 1.0;
pub const MAX_FOV: f32 = std::f32::consts::PI;
pub const MIN_FOV: f32 = 0.7;

fn main() -> Result<(), amethyst::Error> {
    amethyst::start_logger(
    LoggerConfig {
            stdout: StdoutLog::Colored,
            level_filter: LogLevelFilter::Off,
            log_file: None,
            allow_env_override: true,
            log_gfx_backend_level: Some(LogLevelFilter::Off),
            log_gfx_rendy_level: Some(LogLevelFilter::Off),
            module_levels: vec![],
        }
    );
    //let mut game = core::build_application(state::combat_state::CombatState::with_combat(CombatData::basic("Test Combat", Vec::new())))?;
    let mut game = core::build_application(state::menu_state::MainMenuState::default())?;
    //let mut game = core::build_application(MapState::new(None))?;
    game.run();
    Ok(())
}

use std::any::TypeId;

use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use super::*;

define_character!(
    MasterDrone,
    MasterDroneSystem,
    MasterDroneSystemDesc,
    "object/character/master/master.ron",
    "The Avenger",
    "An experimental drone.",
    500.0, // max charge
    500.0, // max health
    500.0,
    0.0, // passive charge per turn
    100.0, // ability charge per turn
    None, // hack modifier (the higher the harder to hack).
    CharacterRole::Master, // Master, Slave, Enemy, Boss,
    Team::Friendly,
    1,
    1.3,
);
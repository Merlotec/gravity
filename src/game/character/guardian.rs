use std::any::TypeId;

use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use super::*;

define_character!(
    GuardianDrone,
    GuardianDroneSystem,
    GuardianDroneSystemDesc,
    "object/character/enemy/guardian/guardian.ron",
    "The Guardian",
    "An AI manufactured superdrone designed to support an antimatter reactor.",
    1500.0, // max charge
    2000.0, // max health
    500.0,
    0.0, // passive charge per turn
    400.0, // ability charge per turn
    None, // hack modifier (the higher the harder to hack).
    CharacterRole::Master, // Master, Slave, Enemy, Boss,
    Team::Enemy,
    1,
    1.3,
);
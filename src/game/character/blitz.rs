use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use super::*;

define_character!(
    BlitzDrone,
    BlitzDroneSystem,
    BlitzDroneSystemDesc,
    "object/character/blitz/blitz.ron",
    "Blitz",
    "An aggressive attack drone.",
    150.0,
    150.0,
    20.0,
    5.0,
    20.0,
    Some(1.0),
    CharacterRole::Slave,
    Team::Friendly,
    1,
    1.0,
);
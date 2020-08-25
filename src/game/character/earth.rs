use std::any::TypeId;

use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use super::*;

define_character!(
    EarthCharacter,
    EarthCharacterSystem,
    EarthCharacterSystemDesc,
    "object/character/earth/earth.ron",
    "Earth",
    "The home planet of the human race. Taken over by AI, there are no longer any (known) humans left on the surface...",
    9999999999.0, // max charge
    9999999999.0, // max health
    9999999999.0,
    0.0, // passive charge per turn
    9999999999.0, // ability charge per turn
    None, // hack modifier (the higher the harder to hack).
    CharacterRole::Independent,
    Team::Enemy,
    1,
    75.0,
);
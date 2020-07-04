use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    NanobotsAbility,
    NanobotsAbilitySystem,
    NanobotsAbilitySystemDesc,
    "Nanobots",
    "Releases a swarm of Nanobots that repair all allies. Can be focused onto one target.",
    30.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::Flexible,
    AbilityUsability::Unique(&[
    DefenderDrone::character_id(),
    SupporterSpacebotDrone::character_id(),
    ModelXDrone::character_id()]),
    [
        AbilityActionDesc::Wave(0, 0.5, None),
       AbilityActionDesc::Heal(75.0, None),
       AbilityActionDesc::Wait(0.7)
    ]
);
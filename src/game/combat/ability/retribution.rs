use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    RetributionAbility,
    RetributionAbilitySystem,
    RetributionAbilitySystemDesc,
    "Retribution",
    "Attacks a single target. The damage output is multiplied by the number of enemy drones.",
    60.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[SpanglesDrone::character_id(),
    SpacebotDrone::character_id(),
    SparkyDrone::character_id(),
    ChargeSpacebotDrone::character_id()
    ]),
    [
       AbilityActionDesc::Fire(
           FireDesc {
                ty: FireType::Laser(Element::Plasma, 1.0, true),
                accuracy: 0.9,
                power: 30.0,
                effect: None,
           },
           0,
       ),
       AbilityActionDesc::Wait(0.7)
    ]
);
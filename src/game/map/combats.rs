use crate::game::map::CombatStore;
use crate::game::combat::{CombatData, CharacterSpawn, Wave, Rank};
use crate::game::character::*;
pub fn combats() -> CombatStore {
    let mut combats: CombatStore = CombatStore::default();
    combats.combat_list.insert(
        "pluto".to_string(),
        CombatData::basic("pluto", "maps/sol/pluto.ron", vec! [
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Basic),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Basic),
            ]),
        ])
    );
    combats.combat_list.insert(
        "neptune".to_string(),
        CombatData::basic("neptune", "maps/sol/neptune.ron",vec! [
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Advanced),
            ]),
        ])
    );
    combats.combat_list.insert(
        "uranus".to_string(),
        CombatData::basic("uranus", "maps/sol/uranus.ron",vec! [
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Advanced),
            ]),
        ])
    );
    combats.combat_list.insert(
        "saturn".to_string(),
        CombatData::basic("saturn", "maps/sol/saturn.ron",vec! [
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Elite),

            ]),
        ])
    );
    combats.combat_list.insert(
        "jupiter".to_string(),
        CombatData::basic("jupiter", "maps/sol/jupiter.ron",vec! [
            Wave::new_simple(vec![
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
            ]),
            Wave::boss(
                CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
            ),
        ])
    );
    combats.combat_list.insert(
        "mars".to_string(),
        CombatData::basic("mars", "maps/sol/mars.ron",vec! [
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
            ]),
        ])
    );
    combats.combat_list.insert(
        "moon".to_string(),
        CombatData::basic("moon", "maps/sol/moon.ron",vec! [
                Wave::boss(
                    CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
                ),
        Wave {
            master: Some(CharacterSpawn::new(GuardianDrone::character_id(), Rank::Basic)),
            characters: vec![
                CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
            ],
        },
        ])
    );
    combats.combat_list.insert(
        "earth".to_string(),
        CombatData::basic("earth", "maps/sol/earth.ron",vec! [
            Wave::boss(
                CharacterSpawn::new(EarthCharacter::character_id(), Rank::Basic),
            ),
        ])
    );

    combats.combat_list.insert(
        "venus".to_string(),
        CombatData::basic("venus", "maps/sol/venus.ron",vec![
            Wave::new_simple(vec![
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Basic),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Elite),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(ChargeSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Advanced),
                CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Advanced),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
            ]),
            Wave::new_simple(vec![
                CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
                CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
            ]),

        ]),
    );

    combats.combat_list.insert(
        "mercury".to_string(),
        CombatData::basic("mercury", "maps/sol/mercury.ron", vec! [
                Wave::new_simple(vec![
                    CharacterSpawn::new(GunnerSpacebotDrone::character_id(), Rank::Elite),
                    CharacterSpawn::new(SupporterSpacebotDrone::character_id(), Rank::Elite),
                    CharacterSpawn::new(SpacebotDrone::character_id(), Rank::Elite),
                ]),

                Wave::new_simple(vec![
                    CharacterSpawn::new(ModelXDrone::character_id(), Rank::Legendary),
                ]),

                Wave::boss(
                    CharacterSpawn::new(GuardianDrone::character_id(), Rank::Legendary),
                ),
        ])
    );
    combats
}
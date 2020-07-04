use std::any::TypeId;

use amethyst::{
    core::{
        Parent,
        ParentHierarchy,
        SystemDesc,
    },
    ecs::prelude::*,
    input::StringBindings,
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::{
        UiButton,
        UiButtonBuilder,
        UiButtonBuilderResources,
        UiEvent,
        UiEventType,
        UiImage,
        UiTransform,
    },
};

use crate::core::{get_root, get_root_cloned};
use crate::game::character::Character;
use crate::game::character::CharacterSpawnError::PrincipalEngaged;
use crate::game::combat::{CombatRoot, Team};
use crate::game::combat::ability::{Ability, AbilityInvoke, AbilityPerform, AbilityTarget, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType};
use crate::game::combat::process::Principal;
use crate::game::ui::crosshair::{CrosshairClickedEvent, CrosshairType, UiCrosshair};
use crate::game::ui::hud::{UiBase, UiCharacterBase};
use crate::game::ui::select_all_button::{SelectAllClickedEvent, UiSelectAllButton};
use crate::game::ui::UiDisengageEvent;
use crate::game::combat::spawn::SlotManager;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SelectAbilityTargetEvent {
    /// An arbitrary entity for which we would like to select the target for.
    /// This can be an ability entity.
    pub ability_ent: Entity,

    /// Set to true to perform this as principal.
    pub as_principal: bool,

    /// Whether to perform the ability on select.
    pub perform_on_select: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AbilityTargetSelectedEvent {
    /// An arbitrary entity for which we would like to select the target for.
    /// This can be an ability entity.
    pub ability_ent: Entity,

    pub target: AbilityTarget,
}

#[derive(Debug, Clone, Component)]
pub struct SelectAbilityTarget {
    ability_ent: Entity,
    available_targets: Vec<Entity>,
    target_all_buttons: Vec<Entity>,
    target_info: AbilityTargetInfo,
    perform_on_select: bool,
}

enum AbilityPerformType {
    Single(Entity),
    Area(Entity, Entity),
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(SelectAbilityTargetSystemDesc))]
pub struct SelectAbilityTargetSystem {
    #[system_desc(event_channel_reader)]
    select_event_reader: ReaderId<SelectAbilityTargetEvent>,

    #[system_desc(event_channel_reader)]
    crosshair_event_reader: ReaderId<CrosshairClickedEvent>,

    #[system_desc(event_channel_reader)]
    select_all_clicked_event_reader: ReaderId<SelectAllClickedEvent>,

    #[system_desc(event_channel_reader)]
    disengage_event_reader: ReaderId<UiDisengageEvent>,
}

impl<'s> System<'s> for SelectAbilityTargetSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, CombatRoot>,
        WriteStorage<'s, SelectAbilityTarget>,
        ReadStorage<'s, UiBase>,
        ReadStorage<'s, Team>,
        ReadStorage<'s, SlotManager>,
        ReadStorage<'s, Ability>,
        WriteStorage<'s, AbilityInvoke>,
        WriteStorage<'s, AbilityPerform>,
        ReadStorage<'s, Character>,
        ReadStorage<'s, UiCharacterBase>,
        WriteStorage<'s, UiCrosshair>,
        WriteStorage<'s, UiSelectAllButton>,
        Read<'s, EventChannel<SelectAbilityTargetEvent>>,
        Read<'s, EventChannel<CrosshairClickedEvent>>,
        Read<'s, EventChannel<SelectAllClickedEvent>>,
        Write<'s, EventChannel<AbilityTargetSelectedEvent>>,
        ReadExpect<'s, ParentHierarchy>,
        Read<'s, EventChannel<UiDisengageEvent>>,
    );

    fn run(&mut self, (entities, mut principals, parents, roots, mut components, ui_bases, teams, slot_managers, abilities, mut invokes, mut performs, characters, character_bases, mut crosshairs, mut select_all_buttons, select_events, crosshair_clicked_events, select_all_clicked_event, mut selected_events, hierarchy, disengage_events): Self::SystemData) {
        let mut to_handle: Vec<AbilityPerformType> = Vec::new();
        for event in crosshair_clicked_events.read(&mut self.crosshair_event_reader) {
            if let Some((target_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &character_bases, event.entity()) {
                to_handle.push(AbilityPerformType::Single(target_base.character_ent));
            }
        }

        for event in select_all_clicked_event.read(&mut self.select_all_clicked_event_reader) {
            // Select all.
            to_handle.push(AbilityPerformType::Area(event.ability_ent, event.team_ent));
        }

        for perform_ty in to_handle {
            let mut perform: Option<(Entity, SelectAbilityTarget, AbilityTarget)> = None;
            match perform_ty {
                AbilityPerformType::Single(target_ent) => {
                    if let Some((select_target, root_ent)) = get_root_cloned::<SelectAbilityTarget, _, _>(&parents, &components, target_ent) {
                        // Ensure we can actually perform this on a single target.
                        if select_target.target_info.area != AbilityTargetArea::All {
                            if let Some((character, target_ent)) = get_root::<Character, _, _>(&parents, &characters, target_ent) {
                                if let Some((team, team_ent)) = Team::get_team(&parents, &teams, target_ent) {
                                    if team.is_target_for(&select_target.target_info.ty) {
                                        perform = Some((root_ent, select_target, AbilityTarget::Single(target_ent)));
                                    }
                                }
                            }
                        }
                    }
                }
                AbilityPerformType::Area(ability_ent, team_ent) => {
                    if let Some((select_target, root_ent)) = get_root_cloned::<SelectAbilityTarget, _, _>(&parents, &components, team_ent) {
                        // Ensure we can actually perform this on multiple targets.
                        if select_target.target_info.area != AbilityTargetArea::Single {
                            let mut target_entities: Vec<Entity> = Vec::new();
                            for (entity, _, _) in (&entities, &characters, hierarchy.all_children(team_ent)).join() {
                                if select_target.available_targets.contains(&entity) {
                                    target_entities.push(entity);
                                }
                            }
                            if !target_entities.is_empty() {
                                perform = Some((root_ent, select_target, AbilityTarget::Multi(target_entities)));
                            }
                        }
                    }
                }
            }

            if let Some((root_ent, select_target, target)) = perform {

                // Remove principal.
                Principal::try_root_disengage(&parents, &mut principals, root_ent, TypeId::of::<Self>());
                if select_target.perform_on_select {
                    if let Some(ability) = abilities.get(select_target.ability_ent) {
                        if Principal::try_root_engage(&parents, &mut principals, select_target.ability_ent, ability.data.system) == Some(true) {
                            performs.insert(select_target.ability_ent, AbilityPerform::new(target));
                            invokes.remove(select_target.ability_ent);
                        }

                    }
                } else {
                    selected_events.single_write(AbilityTargetSelectedEvent {
                        ability_ent: select_target.ability_ent,
                        target,
                    });
                }
                // Remove hover.
                for (crosshair_ent, mut crosshair) in (&entities, &mut crosshairs).join() {
                    if let Some((ui_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &character_bases, crosshair_ent) {
                        if let Some((select_target, character_root)) = get_root::<SelectAbilityTarget, _, _>(&parents, &components, ui_base.character_ent) {
                            if character_root == root_ent {
                                if let Some((team, team_ent)) = Team::get_team(&parents, &teams, ui_base.character_ent) {
                                    crosshair.hover_ty = None;
                                    crosshair.visible = true;
                                }
                            }
                        }
                    }
                }
                for target_all_button_ent in select_target.target_all_buttons.iter() {
                    if let Some(button) = select_all_buttons.get(*target_all_button_ent) {
                        for (ui_base, _) in (&ui_bases, hierarchy.all_children(button.team_ent)).join() {
                            for (mut crosshair, _) in (&mut crosshairs, hierarchy.all_children(ui_base.entity())).join() {
                                crosshair.ty = CrosshairType::Passive(crosshair.ty.team());
                            }
                        }
                    }
                    entities.delete(*target_all_button_ent);
                }
                // Remove root
                components.remove(root_ent);
            }
        }

        for event in select_events.read(&mut self.select_event_reader) {
            if let Some((existing, root_ent)) = get_root::<SelectAbilityTarget, _, _>(&parents, &components, event.ability_ent) {
                // Remove hover.
                for (crosshair_ent, mut crosshair) in (&entities, &mut crosshairs).join() {
                    if let Some((ui_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &character_bases, crosshair_ent) {
                        if let Some((select_target, character_root)) = get_root::<SelectAbilityTarget, _, _>(&parents, &components, ui_base.character_ent) {
                            if character_root == root_ent {
                                if let Some((team, team_ent)) = Team::get_team(&parents, &teams, ui_base.character_ent) {
                                    crosshair.hover_ty = None;
                                    crosshair.visible = true;
                                }
                            }
                        }
                    }
                }
                for target_all_button_ent in existing.target_all_buttons.iter() {
                    entities.delete(*target_all_button_ent);
                }

                // Remove principal.
                Principal::try_root_disengage(&parents, &mut principals, root_ent, TypeId::of::<Self>());
            }
            let mut perform: bool = false;
            if let Some((combat_root, root_ent)) = get_root::<CombatRoot, _, _>(&parents, &roots, event.ability_ent) {
                if let Some(ability) = abilities.get(event.ability_ent) {
                    if let Some(target_info) = ability.data.target_info {
                        if event.as_principal {
                            if Principal::try_root_engage(&parents, &mut principals, root_ent, TypeId::of::<Self>()) == Some(true) {
                                perform = true;
                            }
                        } else {
                            perform = true;
                        }
                        if perform {
                            let target_all_buttons: Vec<Entity> = {
                                let mut result: Vec<Entity> = Vec::new();
                                // Add a select all button if applicable.
                                if target_info.area == AbilityTargetArea::All || target_info.area == AbilityTargetArea::Flexible {
                                    match target_info.ty {
                                        AbilityTargetType::Friendly => {
                                            let button_ent = entities.create();
                                            if let Some(team_ent) = Team::get_local(
                                                &entities,
                                                &parents,
                                                &roots,
                                                &teams,
                                                &hierarchy,
                                                Team::Friendly,
                                                event.ability_ent,
                                            ) {
                                                select_all_buttons.insert(button_ent, UiSelectAllButton::new(event.ability_ent, team_ent, [0.05, 0.1, 0.05, 1.0], [0.0, 1.0, 0.0, 1.0]));
                                                result.push(button_ent);
                                            }
                                        }
                                        AbilityTargetType::Enemy => {
                                            let button_ent = entities.create();
                                            if let Some(team_ent) = Team::get_local(
                                                &entities,
                                                &parents,
                                                &roots,
                                                &teams,
                                                &hierarchy,
                                                Team::Enemy,
                                                event.ability_ent,
                                            ) {
                                                select_all_buttons.insert(button_ent, UiSelectAllButton::new(event.ability_ent, team_ent, [0.1, 0.05, 0.05, 1.0], [1.0, 0.0, 0.0, 1.0]));
                                                result.push(button_ent);
                                            }
                                        }
                                        AbilityTargetType::All => {
                                            let friendly_button_ent = entities.create();
                                            if let Some(team_ent) = Team::get_local(
                                                &entities,
                                                &parents,
                                                &roots,
                                                &teams,
                                                &hierarchy,
                                                Team::Friendly,
                                                event.ability_ent,
                                            ) {
                                                select_all_buttons.insert(friendly_button_ent, UiSelectAllButton::new(event.ability_ent, team_ent, [0.05, 0.1, 0.05, 1.0], [0.0, 1.0, 0.0, 1.0]));
                                                result.push(friendly_button_ent);
                                            }
                                            let enemy_button_ent = entities.create();
                                            if let Some(team_ent) = Team::get_local(
                                                &entities,
                                                &parents,
                                                &roots,
                                                &teams,
                                                &hierarchy,
                                                Team::Enemy,
                                                event.ability_ent,
                                            ) {
                                                select_all_buttons.insert(enemy_button_ent, UiSelectAllButton::new(event.ability_ent, team_ent, [0.1, 0.05, 0.05, 1.0], [0.0, 1.0, 0.0, 1.0]));
                                                result.push(enemy_button_ent);
                                            }
                                        }
                                    }
                                }
                                result
                            };

                            let targets: Vec<Entity> = Ability::targets_for(
                                &entities,
                                &parents,
                                &abilities,
                                &characters,
                                &slot_managers,
                                &teams,
                                event.ability_ent,
                            );

                            components.insert(root_ent, SelectAbilityTarget {
                                ability_ent: event.ability_ent,
                                target_info,
                                target_all_buttons,
                                perform_on_select: event.perform_on_select,
                                available_targets: targets,
                            });

                            // Change crosshairs.
                            for (crosshair_ent, mut crosshair) in (&entities, &mut crosshairs).join() {
                                if let Some((ui_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &character_bases, crosshair_ent) {
                                    if let Some((select_target, character_root)) = get_root::<SelectAbilityTarget, _, _>(&parents, &components, ui_base.character_ent) {
                                        if character_root == root_ent {
                                            if let Some((team, team_ent)) = Team::get_team(&parents, &teams, ui_base.character_ent) {
                                                crosshair.ty = CrosshairType::Passive(team);
                                                if select_target.available_targets.contains(&ui_base.character_ent) {
                                                    crosshair.ty = CrosshairType::Passive(team);
                                                    if target_info.area == AbilityTargetArea::Single || target_info.area == AbilityTargetArea::Flexible {
                                                        crosshair.hover_ty = Some(CrosshairType::Target(team));
                                                    }
                                                    crosshair.visible = true;
                                                } else {
                                                    crosshair.hover_ty = None;;
                                                    crosshair.visible = false
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for event in disengage_events.read(&mut self.disengage_event_reader) {
            if *event == UiDisengageEvent::Cancel {
                let mut to_remove: Vec<Entity> = Vec::new();
                for (root_ent, root, select_target) in (&entities, &roots, &components).join() {
                    // Remove hover.
                    for (crosshair_ent, mut crosshair) in (&entities, &mut crosshairs).join() {
                        if let Some((ui_base, _)) = get_root::<UiCharacterBase, _, _>(&parents, &character_bases, crosshair_ent) {
                            if let Some((_, character_root)) = get_root::<SelectAbilityTarget, _, _>(&parents, &components, ui_base.character_ent) {
                                if character_root == root_ent {
                                    if let Some((team, team_ent)) = Team::get_team(&parents, &teams, ui_base.character_ent) {
                                        crosshair.hover_ty = None;
                                        crosshair.visible = true;
                                    }
                                }
                            }
                        }
                    }
                    for target_all_button_ent in select_target.target_all_buttons.iter() {
                        if let Some(button) = select_all_buttons.get(*target_all_button_ent) {
                            for (ui_base, _) in (&ui_bases, hierarchy.all_children(button.team_ent)).join() {
                                for (mut crosshair, _) in (&mut crosshairs, hierarchy.all_children(ui_base.entity())).join() {
                                    crosshair.ty = CrosshairType::Passive(crosshair.ty.team());
                                }
                            }
                        }
                        entities.delete(*target_all_button_ent);
                    }
                    // Remove root
                    to_remove.push(root_ent);
                    Principal::try_root_disengage(
                        &parents,
                        &mut principals,
                        root_ent,
                        TypeId::of::<Self>(),
                    );
                }
                for ent in to_remove {
                    components.remove(ent);
                }
            }
        }

        for (entity, button) in (&entities, &select_all_buttons).join() {
            for (ui_base, _) in (&ui_bases, hierarchy.all_children(button.team_ent)).join() {
                for (mut crosshair, _) in (&mut crosshairs, hierarchy.all_children(ui_base.entity())).join() {
                    if button.hover {
                        crosshair.ty = CrosshairType::Target(crosshair.ty.team());
                    } else {
                        crosshair.ty = CrosshairType::Passive(crosshair.ty.team());
                    }
                }
            }
        }
    }
}
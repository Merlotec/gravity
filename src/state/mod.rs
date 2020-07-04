use amethyst::{core::{
    ArcThreadPool,
    bundle::SystemBundle,
}, DataDispose, ecs::prelude::*, Error, prelude::*};

use crate::game::combat::CombatData;

pub mod menu_state;
pub mod combat_state;
pub mod map_state;

pub struct AggregateData<'a, 'b> {
    // Dispatchers
    pub game: GameData<'a, 'b>,
    pub combat: GameData<'a, 'b>,
    pub map: GameData<'a, 'b>,
    pub ui: GameData<'a, 'b>,
    pub core: GameData<'a, 'b>,

    // Cross-state data.
    pub combat_invoke: Option<CombatData>,
}

impl<'a, 'b> AggregateData<'a, 'b> {
    pub fn dispatch_all(&mut self, world: &World) {
        self.dispatch_game(world);
        self.dispatch_combat(world);
        self.dispatch_map(world);
        self.dispatch_ui(world);
        self.dispatch_core(world);
    }

    pub fn dispatch_game(&mut self, world: &World) {
        self.game.update(world);
    }

    pub fn dispatch_combat(&mut self, world: &World) {
        self.combat.update(world);
    }

    pub fn dispatch_map(&mut self, world: &World) {
        self.map.update(world);
    }

    pub fn dispatch_ui(&mut self, world: &World) {
        self.ui.update(world);
    }

    pub fn dispatch_core(&mut self, world: &World) {
        self.core.update(world);
    }
}

impl<'a, 'b> DataDispose for AggregateData<'a, 'b> {
    fn dispose(&mut self, world: &mut World) {}
}

/// A 'global' game data builder which holds 'sub' game data for various states.
/// These sub executors can be executed separately to control what groups of systems are run when.
/// This decreases the number of irrelevant system dispatches.
pub struct AggregateDataBuilder<'a, 'b> {
    pub game: GameDataBuilder<'a, 'b>,
    pub combat: GameDataBuilder<'a, 'b>,
    pub map: GameDataBuilder<'a, 'b>,
    pub ui: GameDataBuilder<'a, 'b>,
    pub core: GameDataBuilder<'a, 'b>,
}

impl<'a, 'b> AggregateDataBuilder<'a, 'b> {
    pub fn new() -> Self {
        Self {
            game: GameDataBuilder::new(),
            combat: GameDataBuilder::new(),
            map: GameDataBuilder::new(),
            ui: GameDataBuilder::new(),
            core: GameDataBuilder::new(),
        }
    }
    pub fn with_game(mut self, data: GameDataBuilder<'a, 'b>) -> Self {
        self.game = data;
        self
    }

    pub fn with_combat(mut self, data: GameDataBuilder<'a, 'b>) -> Self {
        self.combat = data;
        self
    }

    pub fn with_map(mut self, data: GameDataBuilder<'a, 'b>) -> Self {
        self.map = data;
        self
    }

    pub fn with_ui(mut self, data: GameDataBuilder<'a, 'b>) -> Self {
        self.ui = data;
        self
    }

    pub fn with_core(mut self, data: GameDataBuilder<'a, 'b>) -> Self {
        self.core = data;
        self
    }
}

impl<'a, 'b> DataInit<AggregateData<'a, 'b>> for AggregateDataBuilder<'a, 'b> {
    fn build(self, world: &mut World) -> AggregateData<'a, 'b> {
        let core = self.core.build(world);
        let ui = self.ui.build(world);
        let game = self.game.build(world);
        let combat = self.combat.build(world);
        let map = self.map.build(world);



        AggregateData { game, combat, ui, core, map, combat_invoke: None }
    }
}
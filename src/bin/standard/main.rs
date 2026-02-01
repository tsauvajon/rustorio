#![forbid(unsafe_code)]

use std::ops::{Add, AddAssign};

use rustorio::{
    self, Bundle, HandRecipe, InsufficientResourceError, Recipe, Resource, ResourceType, Tick,
    buildings::{Assembler, Furnace},
    gamemodes::Standard,
    recipes::{
        CopperSmelting, CopperWireRecipe, ElectronicCircuitRecipe, FurnaceRecipe, IronSmelting,
    },
    resources::{Copper, CopperOre, CopperWire, ElectronicCircuit, Iron, IronOre, Point},
    territory::{Miner, Territory},
};

type GameMode = Standard;

type StartingResources = <GameMode as rustorio::GameMode>::StartingResources;

fn main() {
    rustorio::play::<GameMode>(user_main);
}

fn user_main(mut tick: Tick, starting_resources: StartingResources) -> (Tick, Bundle<Point, 200>) {
    let StartingResources {
        iron,
        mut iron_territory,
        mut copper_territory,
        steel_technology,
    } = starting_resources;

    let iron_furnace = Furnace::build(&tick, IronSmelting, iron);
    assert_eq!(0, iron.amount());

    // Miner
    let iron = SmeltIron {}
        .mine_and_smelt::<10>(&mut tick, iron_territory, iron_furnace)
        .unwrap();
    let copper_furnace = iron_furnace.change_recipe(CopperSmelting).unwrap();
    let copper = SmeltCopper {}
        .mine_and_smelt::<5>(&mut tick, copper_territory, copper_furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    copper_territory.add_miner(&tick, miner);

    // Assembler
    let iron_furnace = copper_furnace.change_recipe(IronSmelting).unwrap();
    let iron = SmeltIron {}
        .mine_and_smelt::<6>(&mut tick, iron_territory, iron_furnace)
        .unwrap();

    // Costs 12 [copper wires](crate::resources::CopperWire) and 6 [iron](crate::resources::Iron).
    // So we need 6 copper
    let copper = SmeltCopper {}
        .mine_and_smelt(&mut tick, copper_territory, copper_furnace)
        .unwrap();
    let mut copper_wires = CopperWireRecipe::craft(&mut tick, (copper,))
        .0
        .to_resource();
    for _ in 0..5 {
        let copper = SmeltCopper {}
            .mine_and_smelt(&mut tick, copper_territory, copper_furnace)
            .unwrap();
        let new_copper_wires = CopperWireRecipe::craft(&mut tick, (copper,))
            .0
            .to_resource();
        copper_wires.add_assign(new_copper_wires);
    }

    let assembler = Assembler::build(
        &tick,
        ElectronicCircuitRecipe,
        copper_wires.bundle().unwrap(),
        iron,
    );
}

trait Smelting {
    type Ore: ResourceType;
    type Smelted: ResourceType;
    type Recipe: FurnaceRecipe;

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Ore>;
    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Smelted>;

    fn mine_and_smelt<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        iron_territory: Territory<Self::Ore>,
        mut furnace: Furnace<Self::Recipe>,
    ) -> Result<Bundle<Self::Smelted, AMOUNT>, InsufficientResourceError<Self::Smelted>> {
        self.hand_mine_into_furnace::<AMOUNT>(tick, iron_territory, &mut furnace);

        // optimise: only ever tick by hand mining, never by just waiting
        tick.advance_until(
            |tick| self.first_output(tick, &mut furnace).amount().ge(&AMOUNT),
            u64::MAX,
        );
        self.first_output(tick, &mut furnace).bundle()
    }

    fn hand_mine_into_furnace<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        mut iron_territory: Territory<Self::Ore>,
        mut furnace: &mut Furnace<Self::Recipe>,
    ) {
        for _ in 0..AMOUNT {
            let iron_ore = iron_territory.hand_mine::<1>(tick);
            self.first_input(tick, &mut furnace).add_bundle(iron_ore);
        }
    }
}

struct SmeltIron;

impl Smelting for SmeltIron {
    type Ore = IronOre;

    type Smelted = Iron;

    type Recipe = IronSmelting;

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Ore> {
        &mut furnace.inputs(tick).0
    }

    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Smelted> {
        &mut furnace.outputs(tick).0
    }
}

struct SmeltCopper;

impl Smelting for SmeltCopper {
    type Ore = CopperOre;

    type Smelted = Copper;

    type Recipe = CopperSmelting;

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Ore> {
        &mut furnace.inputs(tick).0
    }

    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut Furnace<Self::Recipe>,
    ) -> &'a mut Resource<Self::Smelted> {
        &mut furnace.outputs(tick).0
    }
}

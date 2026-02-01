#![forbid(unsafe_code)]

use std::{
    ops::{Add, AddAssign},
    u64,
};

use rustorio::{
    self, Bundle, HandRecipe, InsufficientResourceError, Resource, ResourceType, Tick,
    buildings::{Assembler, Furnace, Lab},
    gamemodes::Standard,
    recipes::{
        CopperSmelting, CopperWireRecipe, ElectronicCircuitRecipe, FurnaceRecipe, IronSmelting,
    },
    resources::{Copper, CopperOre, Iron, IronOre, Point},
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

    // Furnace
    let iron_furnace = Furnace::build(&tick, IronSmelting, iron);
    assert_eq!(0, iron.amount());

    // Iron Miner
    let iron = SmeltIron
        .mine_and_smelt::<10>(&mut tick, iron_territory, iron_furnace)
        .unwrap();
    let copper_furnace = iron_furnace.change_recipe(CopperSmelting).unwrap();
    let copper = SmeltCopper
        .mine_and_smelt::<5>(&mut tick, copper_territory, copper_furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    iron_territory.add_miner(&tick, miner);

    // Copper Miner
    let iron = SmeltIron
        .mine_and_smelt::<10>(&mut tick, iron_territory, iron_furnace)
        .unwrap();
    let copper_furnace = iron_furnace.change_recipe(CopperSmelting).unwrap();
    let copper = SmeltCopper
        .mine_and_smelt::<5>(&mut tick, copper_territory, copper_furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    copper_territory.add_miner(&tick, miner);

    // Dedicated copper furnace
    let iron_ore = iron_territory.resources(&tick).bundle().unwrap();
    let iron_furnace = copper_furnace.change_recipe(IronSmelting).unwrap();
    iron_furnace.inputs(&tick).0.add_bundle(iron_ore);
    tick.advance_until(|_tick| iron_furnace.output_amounts().0 == 10, u64::MAX);
    let iron = iron_furnace.outputs(&tick).0.bundle().unwrap();
    let copper_furnace = Furnace::build(&tick, CopperSmelting, iron);

    // Assembler
    let iron_furnace = copper_furnace.change_recipe(IronSmelting).unwrap();
    let iron = SmeltIron
        .mine_and_smelt::<6>(&mut tick, iron_territory, iron_furnace)
        .unwrap();
    let copper = SmeltCopper
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
    let copper_wires = copper_wires.bundle().unwrap();
    let assembler = Assembler::build(&tick, ElectronicCircuitRecipe, copper_wires, iron);

    // RedScienceRecipe
    // ElectronicCircuitRecipe

    // Lab
    // Lab::build(&tick, &steel_technology, iron, copper);

    // // Dedicated steel furnace
    // let iron_ore = iron_territory.resources(&tick).bundle().unwrap();
    // let iron_furnace = copper_furnace.change_recipe(IronSmelting).unwrap();
    // iron_furnace.inputs(&tick).0.add_bundle(iron_ore);
    // tick.advance_until(|_tick| iron_furnace.output_amounts().0 == 10, u64::MAX);
    // let iron = iron_furnace.outputs(&tick).0.bundle().unwrap();
    // let steel_furnace = Furnace::build(&tick, SteelSmelting, iron);

    // Points farming
}

// TODO: use the Smelting trait instead
fn put_all_ores_in_furnaces(
    tick: &Tick,
    iron_territory: &mut Territory<IronOre>,
    copper_territory: &mut Territory<CopperOre>,
    iron_furnace: &Furnace<IronSmelting>,
    copper_furnace: &Furnace<CopperSmelting>,
) {
    let iron_ore = iron_territory.resources(tick);
    let copper_ore = copper_territory.resources(tick);
    iron_furnace.inputs(tick).0.add(iron_ore);
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
        territory: Territory<Self::Ore>,
        mut furnace: Furnace<Self::Recipe>,
    ) -> Result<Bundle<Self::Smelted, AMOUNT>, InsufficientResourceError<Self::Smelted>> {
        self.mine_into_furnace::<AMOUNT>(tick, territory, &mut furnace);

        // optimise: only ever tick by hand mining, never by just waiting
        tick.advance_until(
            |tick| self.first_output(tick, &mut furnace).amount().ge(&AMOUNT),
            u64::MAX,
        );
        self.first_output(tick, &mut furnace).bundle()
    }

    // optimise: parallelise many miners - this should take care of ticks naturally IMO
    fn mine_into_furnace<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        mut territory: Territory<Self::Ore>,
        mut furnace: &mut Furnace<Self::Recipe>,
    ) {
        for hand_mined in 0..AMOUNT {
            let remaining = AMOUNT - hand_mined;
            let resources = territory.resources(tick);

            if resources.amount().ge(&remaining) {
                let resources = resources.split_off(remaining).unwrap();
                self.first_input(tick, &mut furnace).add(resources);
            }

            let iron_ore = territory.hand_mine::<1>(tick);
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

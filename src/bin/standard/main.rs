#![forbid(unsafe_code)]

use rustorio::{
    self, Bundle, InsufficientResourceError, Resource, ResourceType, Tick,
    buildings::Furnace,
    gamemodes::Standard,
    recipes::{CopperSmelting, FurnaceRecipe, IronSmelting},
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

    let iron_furnace = Furnace::build(&tick, IronSmelting, iron);
    assert_eq!(0, iron.amount());

    let iron = SmeltIron {}
        .mine_and_smelt::<10>(&mut tick, iron_territory, iron_furnace)
        .unwrap();
    let copper_furnace = iron_furnace.change_recipe(CopperSmelting).unwrap();
    let copper = SmeltCopper {}
        .mine_and_smelt::<5>(&mut tick, copper_territory, copper_furnace)
        .unwrap();
    let miner = Miner::build(iron, copper);
    copper_territory.add_miner(&tick, miner);

    // We have 2 furnaces and a miner

    // Costs 12 [copper wires](crate::resources::CopperWire) and 6 [iron](crate::resources::Iron).
    // So we need 24 copper
    // Assembler::build(tick, recipe, copper_wires, iron)
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
        mut iron_territory: Territory<Self::Ore>,
        mut furnace: Furnace<Self::Recipe>,
    ) -> Result<Bundle<Self::Smelted, AMOUNT>, InsufficientResourceError<Self::Smelted>> {
        for _ in 0..AMOUNT {
            let iron_ore = iron_territory.hand_mine::<1>(tick);
            self.first_input(tick, &mut furnace).add_bundle(iron_ore);
        }

        tick.advance_until(
            |tick| self.first_output(tick, &mut furnace).amount().ge(&AMOUNT),
            u64::MAX,
        );
        self.first_output(tick, &mut furnace).bundle()
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

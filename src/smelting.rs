use rustorio::{
    Bundle, Resource, ResourceType, Tick,
    buildings::Furnace,
    recipes::{CopperSmelting, FurnaceRecipe, IronSmelting},
    resources::{Copper, CopperOre, Iron, IronOre},
    territory::Territory,
};
use rustorio_engine::machine::MachineNotEmptyError;

use crate::furnace::{FlexibleFurnace, SupportedFurnaceRecipe};

#[derive(Debug)]
pub enum Error {
    InsufficientResource,
    FurnaceIsNotEmpty,
}

pub trait Smelting {
    type Ore: ResourceType;
    type Smelted: ResourceType;
    type Recipe: FurnaceRecipe + SupportedFurnaceRecipe;

    fn recipe(&self) -> Self::Recipe;

    fn furnace_mut<'a>(&self, furnace: &'a mut FlexibleFurnace) -> &'a mut Furnace<Self::Recipe> {
        furnace
            .as_recipe_mut::<Self::Recipe>()
            .expect("FlexibleFurnace not configured for expected recipe")
    }

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut FlexibleFurnace,
    ) -> &'a mut Resource<Self::Ore>;
    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut FlexibleFurnace,
    ) -> &'a mut Resource<Self::Smelted>;

    fn mine_and_smelt<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        territory: &mut Territory<Self::Ore>,
        furnace: &mut FlexibleFurnace,
    ) -> Result<Bundle<Self::Smelted, AMOUNT>, Error> {
        self.mine_into_furnace::<AMOUNT>(tick, territory, furnace)
            .map_err(|_| Error::FurnaceIsNotEmpty)?;

        // optimise: only ever tick by hand mining, never by just waiting
        tick.advance_until(
            |tick| self.first_output(tick, furnace).amount().ge(&AMOUNT),
            u64::MAX,
        );
        self.first_output(tick, furnace)
            .bundle()
            .map_err(|_| Error::InsufficientResource)
    }

    // optimise: (step1) parallelise many miners - this should take care of ticks naturally IMO
    // optimise: (step2) use a 'yield' system to make progress on all things at once
    fn mine_into_furnace<const AMOUNT: u32>(
        &self,
        tick: &mut Tick,
        territory: &mut Territory<Self::Ore>,
        furnace: &mut FlexibleFurnace,
    ) -> Result<(), MachineNotEmptyError<()>> {
        furnace.change_recipe(self.recipe()).unwrap();

        let mut remaining = AMOUNT;
        for _ in 0..AMOUNT {
            let resources = territory.resources(tick);
            let resources = resources.split_off_max(remaining);

            remaining = remaining - resources.amount();
            self.first_input(tick, furnace).add(resources);

            if remaining == 0 {
                return Ok(());
            }

            let ore = territory.hand_mine::<1>(tick);
            // For manual optimisation
            eprintln!("WARN: hand mining {} at tick {tick}", Self::Ore::NAME);
            remaining = remaining - 1;
            self.first_input(tick, furnace).add_bundle(ore);
        }

        Ok(())
    }
}

// TODO: make it hold the territory and a furnace array so it's easier to call?
pub struct SmeltIron;

impl Smelting for SmeltIron {
    type Ore = IronOre;
    type Smelted = Iron;
    type Recipe = IronSmelting;

    fn recipe(&self) -> Self::Recipe {
        IronSmelting
    }

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut FlexibleFurnace,
    ) -> &'a mut Resource<Self::Ore> {
        &mut self.furnace_mut(furnace).inputs(tick).0
    }

    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut FlexibleFurnace,
    ) -> &'a mut Resource<Self::Smelted> {
        &mut self.furnace_mut(furnace).outputs(tick).0
    }
}

pub struct SmeltCopper;

impl Smelting for SmeltCopper {
    type Ore = CopperOre;
    type Smelted = Copper;
    type Recipe = CopperSmelting;

    fn recipe(&self) -> Self::Recipe {
        CopperSmelting
    }

    fn first_input<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut FlexibleFurnace,
    ) -> &'a mut Resource<Self::Ore> {
        &mut self.furnace_mut(furnace).inputs(tick).0
    }

    fn first_output<'a>(
        &self,
        tick: &Tick,
        furnace: &'a mut FlexibleFurnace,
    ) -> &'a mut Resource<Self::Smelted> {
        &mut self.furnace_mut(furnace).outputs(tick).0
    }
}

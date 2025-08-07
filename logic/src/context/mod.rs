pub mod species;

use std::sync::OnceLock;

use species::SpeciesProvider;

const PKMN: &str = include_str!("../../../reference/pokeapi/data/v2/csv/pokemon_species.csv");
const PKMN_NAMES: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/pokemon_species_names.csv");
const EXP: &str = include_str!("../../../reference/pokeapi/data/v2/csv/experience.csv");
const TYPES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/type_names.csv");
const MOVES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/moves.csv");

pub struct PkContext {
    species: species::SpeciesProvider,
}

impl PkContext {
    fn new() -> PkContext {
        PkContext {
            species: SpeciesProvider::new(),
        }
    }

    pub fn get() -> &'static PkContext {
        static CONTEXT: OnceLock<PkContext> = OnceLock::new();
        CONTEXT.get_or_init(Self::new)
    }

    pub fn species(&self) -> &species::SpeciesProvider {
        &self.species
    }
}

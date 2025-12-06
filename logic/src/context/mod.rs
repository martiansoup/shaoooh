pub mod encounters;
pub mod species;
pub mod versions;

use std::sync::OnceLock;

use encounters::EncountersProvider;
use species::SpeciesProvider;
use versions::VersionsProvider;

use crate::app::Game;

const PKMN_NAMES: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/pokemon_species_names.csv");

const ENCOUNTERS: &str = include_str!("../../../reference/pokeapi/data/v2/csv/encounters.csv");
const ENCOUNTER_SLOTS: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/encounter_slots.csv");
const ENCOUNTER_NAMES: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/encounter_method_prose.csv");
const ENCOUNTER_METHODS: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/encounter_methods.csv");

const ENCOUNTER_CONDITIONS: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/encounter_condition_value_map.csv");
const ENCOUNTER_CONDITION_NAMES: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/encounter_condition_value_prose.csv");

const VERSIONS: &str = include_str!("../../../reference/pokeapi/data/v2/csv/versions.csv");
const VERSION_GROUPS: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/version_groups.csv");

// Names for encounter areas
const AREA_NAMES: &str =
    include_str!("../../../reference/pokeapi/data/v2/csv/location_area_prose.csv");
const AREAS: &str = include_str!("../../../reference/pokeapi/data/v2/csv/location_areas.csv");
const LOC_NAMES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/location_names.csv");
//const PKMN: &str = include_str!("../../../reference/pokeapi/data/v2/csv/pokemon_species.csv");
//const EXP: &str = include_str!("../../../reference/pokeapi/data/v2/csv/experience.csv");
//const TYPES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/type_names.csv");
//const MOVES: &str = include_str!("../../../reference/pokeapi/data/v2/csv/moves.csv");

pub struct PkContext {
    species: species::SpeciesProvider,
    encounters: encounters::EncountersProvider,
    versions: versions::VersionsProvider,
}

impl PkContext {
    fn new() -> PkContext {
        PkContext {
            species: SpeciesProvider::new(),
            encounters: EncountersProvider::new(),
            versions: VersionsProvider::new(),
        }
    }

    pub fn get() -> &'static PkContext {
        static CONTEXT: OnceLock<PkContext> = OnceLock::new();
        CONTEXT.get_or_init(Self::new)
    }

    pub fn species(&self) -> &species::SpeciesProvider {
        &self.species
    }

    pub fn encounters(&self) -> &encounters::EncountersProvider {
        &self.encounters
    }

    pub fn versions(&self) -> &versions::VersionsProvider {
        &self.versions
    }

    pub fn sprite_path(&self, game: &Game, species: u32, shiny: bool) -> String {
        let dir = match game {
            Game::FireRedLeafGreen => "frlg",
            Game::DiamondPearl => "dp",
            Game::RubySapphire => "rs",
            Game::HeartGoldSoulSilver => "hgss",
            Game::Black2White2 => "bw",
            Game::BlackWhite => "bw",
            Game::UltraSunUltraMoon => "usum",
            _ => panic!("Unimplemented game"), // TODO other games
        };
        if shiny {
            format!("../reference/images/{}/{:03}_shiny.png", dir, species)
        } else {
            format!("../reference/images/{}/{:03}.png", dir, species)
        }
    }
}

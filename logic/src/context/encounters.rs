use std::collections::HashMap;

use crate::{
    app::Game,
    context::{self, PkContext},
};

#[derive(Debug)]
struct EncounterUnparsed {
    id: u32,
    version: u32,
    location: u32,
    slot: u32,
    mon: u32,
    min_lvl: u32,
    max_lvl: u32,
}

// TODO derive strings only on debug?
#[derive(Debug)]
struct EncounterParsed {
    id: u32,
    version_id: u32,
    version: String,
    location: String,
    slot: u32,
    method_id: u32,
    method: String,
    rarity: u32,
    mon: u32,
    min_lvl: u32,
    max_lvl: u32,
    condition: String,
    condition_id: u32,
}

pub struct EncountersProvider {}

impl EncountersProvider {
    pub fn new() -> Self {
        Self {}
    }

    fn dedup(mut es: Vec<EncounterUnparsed>) -> Vec<EncounterUnparsed> {
        es.dedup_by(|a, b| {
            (a.version == b.version)
                && (a.location == b.location)
                && (a.slot == b.slot)
                && (a.mon == b.mon)
                && (a.min_lvl == b.min_lvl)
                && (a.max_lvl == b.max_lvl)
        });

        es
    }

    fn parse_encounters(es: Vec<EncounterUnparsed>) -> Vec<EncounterParsed> {
        //let es = Self::dedup(es);
        let mut res = vec![];
        let mut locations = vec![];
        let mut encounter_slots = vec![];
        let mut encounter_ids = vec![];
        let mut location_map = HashMap::new();

        let mut condition_map = HashMap::new();
        let mut condition_names = HashMap::new();
        let mut conditions = vec![];

        let mut encounter_methods = vec![];
        // Slot to rarity
        let mut rarity_map = HashMap::new();
        // Slot to method id
        let mut method_map = HashMap::new();
        // Method id to name
        let mut method_names = HashMap::new();

        for e in &es {
            locations.push(e.location);
            encounter_slots.push(e.slot);
            encounter_ids.push(e.id);
        }

        let mut csv_locs = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::AREA_NAMES.as_bytes());

        for r in csv_locs.records().flatten() {
            let id: u32 = str::parse(&r[0]).expect("Couldn't parse ID");
            let lang: u32 = str::parse(&r[1]).expect("Couldn't parse lang");

            if locations.contains(&id) && lang == 9 {
                let name = r[2].to_string();
                location_map.insert(id, name);
            }
        }

        if location_map.len() != locations.len() {
            let mut needed_areas = vec![];
            let mut needed_locs = vec![];
            let mut area_map = HashMap::new();
            let mut suffix_map = HashMap::new();
            let mut loc_map = HashMap::new();

            // Wasn't in location names, construct from area
            for loc in &locations {
                if !location_map.contains_key(&loc) {
                    needed_areas.push(*loc);
                }
            }

            let mut csv_areas = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(context::AREAS.as_bytes());

            for r in csv_areas.records().flatten() {
                let id: u32 = str::parse(&r[0]).expect("Couldn't parse ID");

                if needed_areas.contains(&id) {
                    let loc_id: u32 = str::parse(&r[1]).expect("Couldn't parse location id");
                    let suffix = &r[3];
                    if suffix.len() > 0 {
                        suffix_map.insert(id, suffix.to_string());
                    }
                    needed_locs.push(loc_id);
                    area_map.insert(id, loc_id);
                }
            }

            let mut csv_loc_name = csv::ReaderBuilder::new()
                .has_headers(true)
                .from_reader(context::LOC_NAMES.as_bytes());

            for r in csv_loc_name.records().flatten() {
                let id = str::parse(&r[0]).expect("Couldn't parse ID");
                let lang: u32 = str::parse(&r[1]).expect("Couldn't parse lang");

                if needed_locs.contains(&id) && lang == 9 {
                    let name = &r[2];
                    loc_map.insert(id, name.to_string());
                }
            }

            for loc in &locations {
                if !location_map.contains_key(&loc) {
                    let area = loc;
                    let location = area_map.get(area).unwrap();
                    let suffix = match suffix_map.get(area) {
                        Some(s) => format!(" {}", s),
                        None => "".to_string(),
                    };
                    let name = match loc_map.get(location) {
                        Some(s) => format!("{}{}", s, suffix),
                        None => "UNKNOWN".to_string(),
                    };
                    location_map.insert(*loc, name);
                }
            }
        }

        let mut csv_slots = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::ENCOUNTER_SLOTS.as_bytes());

        for r in csv_slots.records().flatten() {
            let id = str::parse(&r[0]).expect("Couldn't parse ID");

            if encounter_slots.contains(&id) {
                let method: u32 = str::parse(&r[2]).expect("Couldn't parse method");
                let rarity = str::parse(&r[4]).expect("Couldn't parse rarity");

                method_map.insert(id, method);
                rarity_map.insert(id, rarity);
                encounter_methods.push(method);
            }
        }

        // Not all methods have prose, so do a first pass with short id
        let mut csv_method_init = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::ENCOUNTER_METHODS.as_bytes());

        for r in csv_method_init.records().flatten() {
            let method: u32 = str::parse(&r[0]).expect("Couldn't parse method");

            if encounter_methods.contains(&method) {
                method_names.insert(method, r[1].to_string());
            }
        }

        let mut csv_method = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::ENCOUNTER_NAMES.as_bytes());

        for r in csv_method.records().flatten() {
            let method = str::parse(&r[0]).expect("Couldn't parse method");
            let lang: u32 = str::parse(&r[1]).expect("Couldn't parse lang");

            if encounter_methods.contains(&method) && lang == 9 {
                method_names.insert(method, r[2].to_string());
            }
        }

        let mut csv_cond = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::ENCOUNTER_CONDITIONS.as_bytes());

        for r in csv_cond.records().flatten() {
            let encounter: u32 = str::parse(&r[0]).expect("Couldn't parse ID");

            if encounter_ids.contains(&encounter) {
                let condition: u32 = str::parse(&r[1]).expect("Couldn't parse condition");
                condition_map.insert(encounter, condition);
                conditions.push(condition);
            }
        }

        let mut csv_cond_name = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::ENCOUNTER_CONDITION_NAMES.as_bytes());

        for r in csv_cond_name.records().flatten() {
            let id: u32 = str::parse(&r[0]).expect("Couldn't parse ID");
            let lang: u32 = str::parse(&r[1]).expect("Couldn't parse lang");

            if lang == 9 && conditions.contains(&id) {
                condition_names.insert(id, r[2].to_string());
            }
        }

        for e in es {
            //let location = location_map.get(&e.location).expect("Couldn't find location").to_string();
            // TODO what's missing?
            let location = location_map
                .get(&e.location)
                .map_or("UNKNOWN", |v| v)
                .to_string();

            let method_id = *method_map.get(&e.slot).expect("Couldn't get method");
            let rarity = *rarity_map.get(&e.slot).expect("Couldn't get rarity");
            let method = method_names
                .get(&method_id)
                .expect("Couldn't get method name")
                .clone();

            let condition_id = *condition_map.get(&e.id).unwrap_or(&0);

            let condition = if condition_id == 0 {
                "".to_string()
            } else {
                condition_names
                    .get(&condition_id)
                    .expect("Couldn't get name")
                    .clone()
            };

            let new = EncounterParsed {
                id: e.id,
                version_id: e.version,
                version: PkContext::get().versions().get_name(e.version),
                location,
                slot: e.slot,
                mon: e.mon,
                min_lvl: e.min_lvl,
                max_lvl: e.max_lvl,
                method_id,
                rarity,
                method,
                condition,
                condition_id,
            };

            res.push(new);
        }
        res
    }

    fn get_encounter_list(species: u32, versions: Vec<u32>) -> Vec<EncounterUnparsed> {
        let mut encounters = vec![];
        let mut csv_encounters = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(context::ENCOUNTERS.as_bytes());

        for r in csv_encounters.records().flatten() {
            let id: u32 = str::parse(&r[0]).expect("Couldn't parse ID");
            let version: u32 = str::parse(&r[1]).expect("Couldn't parse version");
            let location: u32 = str::parse(&r[2]).expect("Couldn't parse location");
            let slot: u32 = str::parse(&r[3]).expect("Couldn't parse slot");
            let mon: u32 = str::parse(&r[4]).expect("Couldn't parse species");
            let min_lvl: u32 = str::parse(&r[5]).expect("Couldn't parse min level");
            let max_lvl: u32 = str::parse(&r[6]).expect("Couldn't parse max level");

            if mon == species {
                if versions.contains(&version) {
                    let e = EncounterUnparsed {
                        id,
                        version,
                        location,
                        slot,
                        mon,
                        min_lvl,
                        max_lvl,
                    };

                    encounters.push(e);
                }
            }
        }

        encounters
    }

    pub fn get_encounters(&self, species: u32, game: Option<Game>) {
        let game_str = match &game {
            Some(g) => {
                let s: &'static str = g.into();
                format!(" in {}", s)
            }
            None => "".to_string(),
        };

        let versions = match &game {
            None => PkContext::get().versions().all_ids(),
            Some(game) => PkContext::get().versions().get_ids(game),
        };

        log::trace!("Finding encounters for #{}{}", species, game_str);
        let unparsed = Self::get_encounter_list(species, versions);

        for e in Self::parse_encounters(unparsed) {
            //log::trace!("{:?}", e);
            let name = PkContext::get().species().name(e.mon);
            let condition = if e.condition.len() > 0 {
                format!(" ({})", e.condition)
            } else {
                "".to_string()
            };
            log::trace!(
                "{} can be encountered in {} at {} @ lvl. {}-{} by {} at {}%{}",
                name,
                e.version,
                e.location,
                e.min_lvl,
                e.max_lvl,
                e.method,
                e.rarity,
                condition
            );
        }
    }
    // pub fn new() -> Self {
    //     let mut mapping = HashMap::new();
    //     let mut inv_mapping = HashMap::new();

    //     let mut csv_names = csv::ReaderBuilder::new()
    //         .has_headers(true)
    //         .from_reader(context::PKMN_NAMES.as_bytes());

    //     for r in csv_names.records().flatten() {
    //         let num: u32 = str::parse(&r[0]).expect("Couldn't parse species ID");
    //         let lang: u32 = str::parse(&r[1]).expect("Couldn't parse species ID");
    //         let name = r[2].to_string();

    //         if lang == 9 {
    //             mapping.insert(num, name.clone());
    //             inv_mapping.insert(name.to_lowercase(), num);
    //         }
    //     }

    //     SpeciesProvider { mapping, inv_mapping }
    // }

    // pub fn name(&self, id: u32) -> String {
    //     self.mapping.get(&id).unwrap_or(&"".to_string()).to_string()
    // }

    // pub fn species(&self, name: &str) -> Option<u32> {
    //     let lower = name.to_lowercase();
    //     self.inv_mapping.get(&lower).copied()
    // }
}

impl Default for EncountersProvider {
    fn default() -> Self {
        Self::new()
    }
}

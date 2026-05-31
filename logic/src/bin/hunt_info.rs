use chrono::Datelike;
use shaoooh::context::PkContext;

use simple_logger::SimpleLogger;

use shaoooh::app::Shaoooh;

#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .with_level(log::Level::Debug.to_level_filter())
        .with_utc_timestamps()
        .init()
        .unwrap();

    log::info!("Starting Shaoooh Test : Hunt Information");

    let hunts = Shaoooh::get_all_hunts();
    let mut mons = Vec::new();

    for h in hunts {
        for p in h.phases {
            mons.push((
                p.date,
                p.species,
                h.game.clone(),
                h.method.clone(),
                p.encounters,
            ))
        }
        if let Some(date) = h.date
            && h.complete
        {
            mons.push((date, h.species, h.game, h.method, h.encounters))
        }
    }
    mons.sort_by_key(|f| f.0);
    let mut total_enc = 0;
    let mut num_shiny = 0;
    for m in mons {
        let name = PkContext::get().species().name(m.1);
        log::info!(
            "Caught {} #{} on {}-{}-{} in {:?} in {} encounters",
            name,
            m.1,
            m.0.year(),
            m.0.month(),
            m.0.day(),
            m.2,
            m.4
        );
        total_enc += m.4;
        num_shiny += 1;
    }
    log::info!(
        "Total encounters = {}, {} shinies. Avg. = {}",
        total_enc,
        num_shiny,
        total_enc / num_shiny
    );
}

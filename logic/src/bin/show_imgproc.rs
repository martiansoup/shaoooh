use shaoooh::vision::collection::AvailableProcessing;

use serde_json::Result;

fn main() -> Result<()> {
    let processing = AvailableProcessing::new();

    match processing {
        Ok(processing) => {
            let pstr = serde_json::to_string_pretty(&processing)?;
            println!("{}", pstr);
        }
        Err(e) => {
            println!("ERROR {:?}", e)
        }
    }

    Ok(())
}

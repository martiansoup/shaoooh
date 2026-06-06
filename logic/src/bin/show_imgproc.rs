use shaoooh::vision::collection::AvailableProcessing;

use serde_json::Result;

fn main() -> Result<()> {
    let processing = AvailableProcessing::new();

    if let Ok(processing) = processing {
        let pstr = serde_json::to_string_pretty(&processing)?;

        println!("{}", pstr);
    }

    Ok(())
}

use sonda_core::classify::outcome::ClassificationResult;
use sonda_core::error::SondaError;

pub fn print(result: &ClassificationResult) -> Result<(), SondaError> {
    let json = serde_json::to_string_pretty(result)?;
    println!("{json}");
    Ok(())
}

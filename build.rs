use std::error::Error;
use vergen::EmitBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // For getting version information with vergen
    EmitBuilder::builder().all_git().emit()?;
    Ok(())
}

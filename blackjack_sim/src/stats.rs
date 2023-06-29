use crate::SimulationSummary;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::sync::mpsc::Receiver;

/// A public function to take in data i.e. `summary` a `SimulationSummary` object and write it to a writer
pub fn write(
    receiver: Receiver<(SimulationSummary, usize)>,
    writer: impl Write,
) -> Result<(), Box<dyn Error>> {
    let mut summaries: HashMap<usize, SimulationSummary> = HashMap::new();
    Ok(())
}

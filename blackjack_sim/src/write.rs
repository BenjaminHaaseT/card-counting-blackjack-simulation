use crate::SimulationSummary;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::Write;
use std::iter::FromIterator;
use std::sync::mpsc::Receiver;

fn format_summaries(summaries: HashMap<usize, SimulationSummary>) -> HashMap<usize, String> {
    const width: usize = 80;
    const text_width: usize = "number of player blackjacks".len() + 20;
    const num_width: usize = width - text_width;
    summaries
        .into_iter()
        .map(|(id, summary)| {
            let sim_num = format!("simulation #{}", id);
            let header = format!("{:-^width$}\n", sim_num);
            (id, format!("{}{}{}\n", header, summary, "-".repeat(width)))
        })
        .collect::<HashMap<usize, String>>()
}

/// A public function to take in data i.e. `summary` a `SimulationSummary` object and write it to a writer
pub fn write(
    receiver: Receiver<(Option<SimulationSummary>, usize)>,
    mut ids: HashSet<usize>,
    mut writer: impl Write,
) -> std::io::Result<()> {
    let mut summaries: HashMap<usize, SimulationSummary> = HashMap::new();
    loop {
        let (cur_summary, id) = receiver.recv().unwrap();
        if let Some(cur_sum) = cur_summary {
            if let Some(summary) = summaries.get_mut(&id) {
                summary.wins += cur_sum.wins;
                summary.pushes += cur_sum.pushes;
                summary.losses += cur_sum.losses;
                summary.winnings += cur_sum.winnings;
                summary.player_blackjacks += cur_sum.player_blackjacks;
                summary.early_endings += cur_sum.early_endings;
            } else {
                summaries.insert(id, cur_sum);
            }
        } else {
            ids.remove(&id);
            if ids.is_empty() {
                // We have no more stats to process
                break;
            }
        }
    }

    // Get summaries into nicely formatted strings, and write to writer
    let formatted_summaries = format_summaries(summaries);
    for i in 1..=formatted_summaries.len() {
        writer.write(formatted_summaries[&i].as_bytes())?;
    }
    Ok(())
}

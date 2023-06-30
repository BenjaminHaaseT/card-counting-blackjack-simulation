use crate::SimulationSummary;
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::sync::mpsc::Receiver;

fn format_summaries(summaries: HashMap<usize, SimulationSummary>) -> impl Iterator<Item = String> {
    const width: usize = 80;
    const text_width: usize = "number of player blackjacks".len() + 20;
    const num_width: usize = width - text_width;
    summaries.into_iter().map(|(id, summary)| {
        format!(
            "{0:-^width$}\n\
            {:-width$}\n\
            {1:<text_width$}{2:>num_width$}\n\
            {3:<text_width$}{4:>num_width$}\n\
            {5:<text_width$}{6:>num_width$}\n\
            {7:<text_width$}{8:>num_width$.2}\n\
            {9:<text_width$}{10:>num_width$}\n\
            {11:<text_width$}{12:>num_width$}\n",
            "simulation #{id}",
            "hands won",
            summary.wins,
            "hands pushed",
            summary.pushes,
            "hands lost",
            summary.losses,
            "winnings",
            summary.winnings,
            "number of player blackjacks",
            summary.player_blackjacks,
            "number of early endings",
            summary.early_endings,
        )
    })
}

/// A public function to take in data i.e. `summary` a `SimulationSummary` object and write it to a writer
pub fn write(
    receiver: Receiver<(SimulationSummary, usize)>,
    mut writer: impl Write,
) -> std::io::Result<()> {
    let mut summaries: HashMap<usize, SimulationSummary> = HashMap::new();
    for (cur_summary, id) in receiver.iter() {
        if let Some(summary) = summaries.get_mut(&id) {
            summary.wins += cur_summary.wins;
            summary.pushes += cur_summary.pushes;
            summary.losses += cur_summary.losses;
            summary.winnings += cur_summary.winnings;
            summary.player_blackjacks += cur_summary.player_blackjacks;
            summary.early_endings += cur_summary.early_endings;
        } else {
            summaries.insert(id, cur_summary);
        }
    }
    // Write data to writer
    for summary_str in format_summaries(summaries) {
        writer.write(summary_str.as_bytes())?;
    }
    Ok(())
}

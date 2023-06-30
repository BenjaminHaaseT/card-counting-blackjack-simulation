use blackjack_sim::strategy::{
    BasicStrategy, CountingStrategy, HiLo, MarginBettingStrategy, PlayerStrategy, WongHalves, KO,
};
use blackjack_sim::{
    BlackjackSimulatorConfig, BlackjackSimulatorConfigBuilder, MulStrategyBlackjackSimulator,
    MulStrategyBlackjackSimulatorBuilder,
};

fn main() {
    let mut simulator = MulStrategyBlackjackSimulator::new(BlackjackSimulatorConfig::default())
        .simulation(PlayerStrategy::new(
            HiLo::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .build();
    let _ = simulator.run();
}

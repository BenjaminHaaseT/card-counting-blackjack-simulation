use blackjack_sim::strategy::{
    BasicStrategy, CountingStrategy, HiLo, HiOptI, HiOptII, MarginBettingStrategy, PlayerStrategy,
    RedSeven, WongHalves, KO,
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
        .simulation(PlayerStrategy::new(
            WongHalves::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            KO::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            RedSeven::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            HiOptI::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            HiOptII::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .build();
    let _ = simulator.run();
}

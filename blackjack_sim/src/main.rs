use blackjack_sim::strategy::{
    AceFive, BasicStrategy, CountingStrategy, Halves, HiLo, HiOptI, HiOptII, MarginBettingStrategy,
    OmegaII, PlayerStrategy, RedSeven, WongHalves, ZenCount, KISS, KISSII, KISSIII, KO,
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
        .simulation(PlayerStrategy::new(
            AceFive::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            OmegaII::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            ZenCount::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            Halves::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            KISS::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            KISSII::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .simulation(PlayerStrategy::new(
            KISSIII::new(6),
            BasicStrategy::new(),
            MarginBettingStrategy::new(3.0, 5),
        ))
        .build();
    let _ = simulator.run();
}

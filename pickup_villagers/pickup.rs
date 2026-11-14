fn pickup_villagers_pickup(game: &mut crate::game::Game<'_>, index: usize) -> core::ops::ControlFlow<bool> {
    crate::game::lan_trading::pickup_villagers::pick_up(game, index)
}

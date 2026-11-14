fn pickup_villagers_place(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
) -> core::ops::ControlFlow<Result<bool>> {
    crate::game::lan_trading::pickup_villagers::place(game, stack)
}

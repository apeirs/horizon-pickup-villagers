fn pickup_villagers_send_components(
    item_id: &dyn Fn(&str) -> i32,
    synced_id: &dyn Fn(&str, &str) -> Option<i32>,
    s: &Option<crate::items::ItemStack>,
) -> core::ops::ControlFlow<Option<Stack>> {
    crate::lan::guest::pickup_villagers_components::stack(item_id, synced_id, s)
}

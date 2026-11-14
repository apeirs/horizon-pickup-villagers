// The item components vanilla's server sends that Horizon's encoder leaves out:
// custom data, lore and a player head's profile. A client then shows them, and a
// stack it sends back (the creative inventory) keeps them.
use crate::items::ItemStack;
use horizon_protocol::item::{Component, Profile, SkinPatch, Stack, component};
use horizon_world::nbt::Tag;
use std::ops::ControlFlow;

pub(crate) fn stack(
    item_id: &dyn Fn(&str) -> i32,
    synced_id: &dyn Fn(&str, &str) -> Option<i32>,
    s: &Option<ItemStack>,
) -> ControlFlow<Option<Stack>> {
    let mut out = super::network_stack(item_id, synced_id, s);
    if let (Some(stack), Some(data)) = (out.as_mut(), s.as_ref().and_then(|s| s.data())) {
        let custom = data.other("minecraft:custom_data").cloned().map(Component::Nbt);
        add(stack, component::CUSTOM_DATA, custom);
        add(stack, component::LORE, data.other("minecraft:lore").and_then(lore));
        add(stack, component::PROFILE, data.other("minecraft:profile").and_then(profile));
    }
    ControlFlow::Break(out)
}

fn add(stack: &mut Stack, kind: i32, value: Option<Component>) {
    if let Some(value) = value
        && !stack.added.iter().any(|(k, _)| *k == kind)
    {
        stack.added.push((kind, value));
    }
}

fn text(t: &Tag) -> Option<String> {
    match t {
        Tag::String(s) => Some(s.to_string_lossy()),
        _ => None,
    }
}

/// `ItemLore`: its lines, each a text component.
fn lore(t: &Tag) -> Option<Component> {
    match t {
        Tag::List(lines) if !lines.is_empty() => Some(Component::Lore(lines.clone())),
        _ => None,
    }
}

/// `ResolvableProfile`: a name alone, or a compound of name, id, properties and
/// `PlayerSkin.Patch` (`texture`, `cape`, `elytra`, `model`).
fn profile(t: &Tag) -> Option<Component> {
    if let Some(name) = text(t) {
        return Some(Component::Profile(Profile {
            name: Some(name),
            id: None,
            properties: Vec::new(),
            skin: SkinPatch::default(),
        }));
    }
    let Tag::Compound(_) = t else { return None };
    let id = match t.get("id") {
        Some(Tag::IntArray(w)) if w.len() == 4 => Some(
            w.iter()
                .fold(0u128, |id, &v| (id << 32) | u128::from(v as u32)),
        ),
        _ => None,
    };
    let properties = match t.get("properties") {
        Some(Tag::List(list)) => list
            .iter()
            .filter_map(|p| {
                Some((
                    text(p.get("name")?)?,
                    text(p.get("value")?)?,
                    p.get("signature").and_then(text),
                ))
            })
            .collect(),
        _ => Vec::new(),
    };
    let skin = SkinPatch {
        body: t.get("texture").and_then(text),
        cape: t.get("cape").and_then(text),
        elytra: t.get("elytra").and_then(text),
        slim: t.get("model").and_then(text).map(|m| m == "slim"),
    };
    Some(Component::Profile(Profile {
        name: t.get("name").and_then(text),
        id,
        properties,
        skin,
    }))
}

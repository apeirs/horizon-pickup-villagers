// Pickup Villagers for Horizon, a port of ClickVillagers' pickup by Clickism
// (https://github.com/Clickism/ClickVillagers, GPL-3.0): sneak and right click a
// villager (or a zombie villager) to pick it up as a head holding its whole
// entity; right click a block with the head to place it back. The heads are
// ClickVillagers' own (`custom_data` with `EntityType` and `CVDataVersion`), so
// they carry over to and from Java.
//
// Copyright (C) Clickism (ClickVillagers) and apeirs (this port).
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3. It comes WITHOUT ANY WARRANTY; see LICENSE.
use super::super::{Game, native_hooks::NativeMod};
use super::pickup_villager_heads as heads;
use horizon_client::{
    first_person::Hand,
    items::ItemStack,
    mobs::{Mob, Species, render::PROFESSIONS},
};
use horizon_protocol::packets::play;
use horizon_world::nbt::Tag;
use std::{ops::ControlFlow, time::Instant};
use winit::keyboard::KeyCode;

pub(crate) const ID: &str = "pickup_villagers";
const HEAD: &str = "minecraft:player_head";
/// ClickVillagers' keys in the head's custom data.
const TYPE_KEY: &str = "EntityType";
const VERSION_KEY: &str = "CVDataVersion";

pub(crate) fn list(mods: &mut Vec<NativeMod>) {
    mods.push(NativeMod {
        id: ID.to_owned(),
        title: "Pickup Villagers 1.0.0".to_owned(),
        description: "Sneak and right click a villager to pick it up; right click a block \
                      with it to place it back."
            .to_owned(),
        keys: Vec::new(),
    });
}

/// A line over the hotbar (`sendSystemMessage(text, true)`), to whoever acts.
fn tell(game: &mut Game<'_>, text: &str) {
    if game.lan_acting.is_some() {
        game.guest_packets
            .push(play::Clientbound::SystemChat(play::SystemChat {
                content: horizon_protocol::text::literal(text),
                overlay: true,
            }));
    } else {
        game.message = Some((text.to_owned(), Instant::now()));
    }
}

// --- ClickVillagers --------------------------------------------------------


/// `PickupVillagerListener`: a sneaking main-hand click on a villager or a
/// zombie villager takes it as a head (into the hand when it is empty).
pub(crate) fn pick_up(game: &mut Game<'_>, index: usize) -> ControlFlow<bool> {
    if game.remote.is_some()
        || !game.native_on(ID)
        || game.hand != Hand::Main
        || !game.keys.contains(&KeyCode::ShiftLeft)
        || game.spectator()
        || game.held().is_some_and(|h| &*h.item == "minecraft:name_tag")
    {
        return ControlFlow::Continue(());
    }
    let m = &game.mobs[index];
    let zombie = m.species() == Species::ZombieVillager;
    // A villager trading with someone stays where it is.
    if !(zombie || m.species() == Species::Villager) || !m.alive() || (!zombie && m.extra.flag2) {
        return ControlFlow::Continue(());
    }
    let (id, at) = (m.id, [m.position.x, m.position.y, m.position.z]);
    // Leashable.dropLeash, out of its boat or minecart, then saved and discarded.
    game.break_leash(id, true, false);
    game.mobs[index].extra.mount = None;
    game.mobs[index].extra.passenger = false;
    let head = head_item(game, index);
    game.mobs[index].removed = true;
    if game.held().is_none() {
        *game.held_mut() = Some(head);
    } else if let Some(rest) = game.give(head) {
        let eye = game.eye_position().unwrap_or(at);
        game.spawn_item(rest, eye, [0.0; 3], 40);
    }
    game.audio
        .play("minecraft:entity.item.pickup", Some(at), 0.6, 0.8);
    tell(game, "You picked up a villager.");
    game.swing_arm();
    ControlFlow::Break(true)
}

/// `PlaceVillagerListener`: a villager head used on a block puts its villager
/// on the face clicked; the head is used up, in creative too (ClickVillagers
/// clears the slot).
pub(crate) fn place(game: &mut Game<'_>, stack: &ItemStack) -> ControlFlow<horizon_client::Result<bool>> {
    if game.remote.is_some() || &*stack.item != HEAD || !game.native_on(ID) {
        return ControlFlow::Continue(());
    }
    let Some(mut tag) = stack
        .data()
        .and_then(|d| d.other("minecraft:custom_data"))
        .filter(|t| t.get(TYPE_KEY).is_some())
        .cloned()
    else {
        return ControlFlow::Continue(());
    };
    let Some(hit) = game.pick() else {
        return ControlFlow::Continue(());
    };
    let p = [
        hit.pos[0] + hit.face[0],
        hit.pos[1] + hit.face[1],
        hit.pos[2] + hit.face[2],
    ];
    let at = [f64::from(p[0]) + 0.5, f64::from(p[1]), f64::from(p[2]) + 0.5];
    // removeUuidIfDuplicate: a copied head brings a second villager.
    let duplicate = horizon_client::entity_nbt::uuid(&tag)
        .is_some_and(|uuid| game.persist.uuids.values().any(|u| *u == uuid));
    if let Tag::Compound(c) = &mut tag {
        let kind = c.remove(&TYPE_KEY.into());
        c.remove(&VERSION_KEY.into());
        if !c.contains_key(&"id".into())
            && let Some(kind) = kind
        {
            c.insert("id".into(), kind);
        }
        if duplicate {
            c.remove(&"UUID".into());
        }
        c.insert("Pos".into(), Tag::List(at.iter().map(|&v| Tag::Double(v.to_bits())).collect()));
        c.insert("Motion".into(), Tag::List(vec![Tag::Double(0); 3]));
        c.insert("fall_distance".into(), Tag::Double(0));
    }
    let zombie = horizon_client::entity_nbt::kind(&tag).is_some_and(|k| k.ends_with("zombie_villager"));
    if game.load_mob(tag, None).is_none() {
        tell(game, "Couldn't read villager data.");
        return ControlFlow::Break(Ok(true));
    }
    // `consume_held` spares creative: one head, one villager.
    if let Some(h) = game.held_mut().as_mut() {
        h.count -= 1;
        if h.count == 0 {
            *game.held_mut() = None;
        }
    }
    let sound = if zombie {
        "minecraft:entity.zombie_villager.ambient"
    } else {
        "minecraft:entity.villager.ambient"
    };
    game.audio.play(sound, Some(at), 1.0, 1.0);
    game.swing_arm();
    ControlFlow::Break(Ok(true))
}

/// `PickupHandler.toItemStack`: a player head named after the villager, its
/// trades as lore, the entity in custom data.
fn head_item(game: &mut Game<'_>, index: usize) -> ItemStack {
    let mut data = game.mob_tree_nbt(index);
    let m = &game.mobs[index];
    if let Tag::Compound(c) = &mut data {
        c.insert(TYPE_KEY.into(), Tag::String(m.kind.id.as_str().into()));
        c.insert(
            VERSION_KEY.into(),
            Tag::Int(horizon_core::version::DATA_VERSION),
        );
    }
    let mut lore = vec![line("Right click to place the villager back.", "dark_gray")];
    let offers = m
        .extra
        .trade
        .as_ref()
        .filter(|t| !t.unset)
        .map(|t| t.offers.clone())
        .unwrap_or_default();
    if !offers.is_empty() {
        lore.push(line(" Trades:", "gold"));
        for o in &offers {
            let mut s = format!("  {}", stack_text(&o.base_cost_a.stack));
            if let Some(b) = &o.cost_b {
                s.push_str(&format!(" + {}", stack_text(&b.stack)));
            }
            s.push_str(&format!(" \u{2192} {}", stack_text(&o.result)));
            lore.push(line(&s, "gray"));
        }
    }
    let components = Tag::compound([
        ("minecraft:custom_data", data),
        ("minecraft:item_name", styled(&display_name(m), "yellow")),
        ("minecraft:lore", Tag::List(lore)),
        ("minecraft:profile", profile(texture(m))),
    ]);
    let (damage, extra) = horizon_client::item_data::decode_components(Some(&components));
    ItemStack {
        item: HEAD.into(),
        count: 1,
        damage,
        extra,
    }
}

fn profession(m: &Mob) -> &'static str {
    PROFESSIONS[m.extra.variant2 as usize % PROFESSIONS.len()]
}

/// `PickupHandler.getDisplayName`.
fn display_name(m: &Mob) -> String {
    if let Some(name) = &m.custom_name {
        return name.clone();
    }
    if m.species() == Species::ZombieVillager {
        return "Zombie Villager".to_owned();
    }
    if m.is_baby() {
        return "Baby Villager".to_owned();
    }
    match profession(m) {
        "none" => "Villager".to_owned(),
        p => format!("{} Villager", title_case(p)),
    }
}

/// `VillagerTextures.getTexture`.
fn texture(m: &Mob) -> &'static str {
    if m.species() == Species::ZombieVillager {
        return heads::ZOMBIE;
    }
    if m.is_baby() {
        return heads::BABY;
    }
    let p = profession(m);
    heads::PROFESSIONS
        .iter()
        .find(|(name, _)| *name == p)
        .map_or(heads::DEFAULT, |(_, t)| t)
}

fn profile(texture: &str) -> Tag {
    Tag::compound([(
        "properties",
        Tag::List(vec![Tag::compound([
            ("name", Tag::String("textures".into())),
            ("value", Tag::String(texture.into())),
        ])]),
    )])
}

fn styled(text: &str, color: &str) -> Tag {
    Tag::compound([
        ("text", Tag::String(text.into())),
        ("color", Tag::String(color.into())),
        ("italic", Tag::Byte(0)),
    ])
}

fn line(text: &str, color: &str) -> Tag {
    styled(text, color)
}

/// `24 Paper`, `Enchanted Book (Mending)`.
fn stack_text(s: &ItemStack) -> String {
    let mut out = item_name(&s.item);
    if s.count > 1 {
        out = format!("{} {out}", s.count);
    }
    let enchants: Vec<String> = s
        .stored_enchantments()
        .iter()
        .chain(s.enchantments().iter())
        .map(|(id, level)| {
            let path = id.trim_start_matches("minecraft:");
            let name = horizon_client::lang::get_or(
                &format!("enchantment.minecraft.{path}"),
                &title_case(path),
            );
            let max_one = matches!(path, "mending" | "silk_touch" | "infinity" | "flame" | "channeling" | "multishot" | "aqua_affinity");
            if max_one && *level == 1 {
                name
            } else {
                format!("{name} {}", roman(*level))
            }
        })
        .collect();
    if !enchants.is_empty() {
        out.push_str(&format!(" ({})", enchants.join(", ")));
    }
    out
}

fn item_name(id: &str) -> String {
    let path = id.trim_start_matches("minecraft:");
    let item = horizon_client::lang::get_or(&format!("item.minecraft.{path}"), "");
    if !item.is_empty() {
        return item;
    }
    horizon_client::lang::get_or(&format!("block.minecraft.{path}"), &title_case(path))
}

fn roman(level: u32) -> String {
    match level {
        1 => "I".to_owned(),
        2 => "II".to_owned(),
        3 => "III".to_owned(),
        4 => "IV".to_owned(),
        5 => "V".to_owned(),
        n => n.to_string(),
    }
}

fn title_case(id: &str) -> String {
    id.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().chain(c).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

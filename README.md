# Pickup Villagers for Horizon

A native mod for [Horizon](https://github.com/horizon-rust/horizon), a port of the pickup feature of
[ClickVillagers](https://github.com/Clickism/ClickVillagers) by Clickism: carry villagers around in your inventory.

- **Pick up:** sneak and right click a villager or a zombie villager with your main hand. It becomes a head named after
  it (`Librarian Villager`, `Baby Villager`, its name tag's name), with its trades listed in the tooltip. A villager
  that is trading with someone stays where it is.
- **Place:** right click a block with the head. The villager comes back on that face exactly as it was: profession,
  level, trades, experience, gossip, inventory, name. A copied head (creative) brings a second villager with a new
  UUID.
- The heads are ClickVillagers' own (`custom_data` with `EntityType` and `CVDataVersion`), so they carry over to and
  from a Java world that runs ClickVillagers.
- Single player and LAN worlds you host. Players who join your LAN world see the heads' names, lore and skins too.

## Install

Native mods are compiled into the game by the Horizon launcher and run with your account's rights: read the source
first.

- From the launcher: Mods, then **Add from GitHub**, `https://github.com/apeirs/horizon-pickup-villagers`, then Build.
- From the command line: `horizon-launcher mods add-github https://github.com/apeirs/horizon-pickup-villagers --trust-native-code`.

## Build and package

The bundle is `pickup_villagers/` (`horizon-mixin.toml` and its `.rs` files). `horizon-launcher native-build
pickup_villagers --trust-native-code` builds it into the selected instance. `python3 tools/package.py --horizon
<commit> --mixins <commit>` writes `dist/pickup_villagers-<version>.hnmod`, pinned to the Horizon commit it was tested
on, for a release.

## Licence and credits

A port of ClickVillagers by Clickism (GPL-3.0); the villager head textures are ClickVillagers' (`VillagerTextures`).
This port is licensed under the GNU General Public License version 3 (LICENSE), as the original is. Not made or
endorsed by Clickism or the Horizon project.

#!/usr/bin/env python3
"""Package the bundle as a .hnmod for a GitHub release and the Horizon registry.

    python3 tools/package.py --horizon <commit> --mixins <commit>

The bundle folder keeps no pin, so a local `horizon-launcher native-build <folder>` builds it on the Horizon the
instance plays. The .hnmod pins the commit it was built and tested on: `[compatibility] horizon_revision` in its
horizon-mixin.toml and `horizon` in horizon-native.json, as the registry's checks want. The archive is
deterministic (sorted names, fixed times), so the same sources give the same SHA-256.
"""
import argparse
import hashlib
import json
import re
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--horizon", required=True, help="the Horizon commit it was built and tested on (40 hex)")
    parser.add_argument("--mixins", required=True, help="the horizon-mixins commit that build used (40 hex)")
    parser.add_argument("--game", default="26.3")
    args = parser.parse_args()
    for rev in (args.horizon, args.mixins):
        if not re.fullmatch(r"[0-9a-f]{40}", rev):
            raise SystemExit(f"not a full commit: {rev}")
    bundle = next(p.parent for p in ROOT.glob("*/horizon-mixin.toml"))
    text = (bundle / "horizon-mixin.toml").read_text()
    manifest = tomllib.loads(text)
    if "compatibility" in manifest:
        raise SystemExit("the source keeps no pin: take [compatibility] out of horizon-mixin.toml")
    mod_id, version = manifest["mod"]["id"], manifest["mod"]["version"]
    if bundle.name != mod_id:
        raise SystemExit(f"the bundle folder {bundle.name} is not named after its id {mod_id}")
    pinned = text.replace(
        "\n[[", f'\n# The commit this release was built and tested on.\n[compatibility]\nhorizon_revision = "{args.horizon}"\n\n[[', 1
    )
    files = {
        "horizon-native.json": (json.dumps({"schema": 1, "id": mod_id, "version": version, "game": args.game,
                                            "horizon": args.horizon, "mixins": args.mixins},
                                           indent=2, sort_keys=True) + "\n").encode(),
        f"mixins/{mod_id}/horizon-mixin.toml": pinned.encode(),
        f"mixins/{mod_id}/LICENSE": (ROOT / "LICENSE").read_bytes(),
    }
    for path in sorted(bundle.rglob("*")):
        if path.is_file() and path.suffix == ".rs":
            files[f"mixins/{mod_id}/{path.relative_to(bundle).as_posix()}"] = path.read_bytes()
    out = ROOT / "dist" / f"{mod_id}-{version}.hnmod"
    out.parent.mkdir(exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    data = out.read_bytes()
    print(json.dumps({"file": str(out), "size": len(data), "sha256": hashlib.sha256(data).hexdigest()}, indent=2))


if __name__ == "__main__":
    main()

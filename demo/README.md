# Demo: Beispielprojekt

Generated, do not edit by hand: `world.json`, `beispiel-de.klappe` and `beispiel-en.klappe`.

- `world.json` is the demo world "Studio Weber", copied by `python3 demo/tools/sync-demo.py klappe`
  in `shrippen.github.io`. The example is its `klappe` section: scene 14 of *Nachtwache* in the
  Graufeld lighthouse. Change the world there, never here.
- `beispiel-de.klappe`, `beispiel-en.klappe` are made from it by `cargo run -p klappe-demo`
  (through `klappe-store`, so in the format the app saves). A test in `crates/demo` (part of
  `make check`) fails when they are out of date.
- `services::copy_example` copies one of them to a folder; the app embeds both (`crates/services/build.rs`).
- `start.sh [de|en]` starts the app on a copy of the example with its own settings and folders;
  with `SHOT_DIR` and `SHOT_PLAN` it takes the shots of `shots.json` offscreen
  (`demo/tools/screenshots.py` in `shrippen.github.io`). Screenshots are taken for a tagged release only.

Only the two `.klappe` folders ship (inside the app). `start.sh`, `shots.json` and `world.json` never do.
`testdata/beispiel.klappe` is the test fixture and stays as it is.

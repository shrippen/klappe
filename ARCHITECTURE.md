# Architecture

How this repository is meant to be built. Where the code differs today is listed under
"Known deviations". Change this file in the PR that changes the structure.

## What it does

Klappe is a film pre-production suite for directors and their crew. Its first module,
Stellprobe, plans a scene on a floor plan (blocking, cameras, light, the 180° axis, shot
list). The screenplay is written as Fountain both in the window and in the terminal. Every
module works on one project, a folder of text files.

## Building blocks

Each block is one crate under `crates/`, unless the path says otherwise.

| Path | Its one task |
|---|---|
| `app/` | The desktop window, the screenplay editor included: QML pages, and the bridge that hands them the session's data and turns the display list into Kante shapes. |
| `app/qml/Kante` | Copy of the Kante design system (`scripts/sync-kante.sh`); never edited here. |
| `tui/` | The terminal program for writing the screenplay, beside the window's editor. |
| `cli/` | The command line: exports and checks without a window. |
| `services/` | The project session, the one door for every surface: open, select, edit, undo, play, save, export. Hands out data ready to show and holds all texts (`i18n/`). |
| `stellprobe/` | Stellprobe's calculations, without files or window: who stands where at a time, what a camera sees, the axis, the display list (what a plan shows, in metres, colours as roles), and the set as bodies (`scene`) with the picture a view of them gives (`picture`, through `raster/`). |
| `raster/` | Fills triangles on the screen into a frame with depth: role, shade, source and edges per pixel, no colours; knows nothing of sets; depends on nothing. |
| `script/` | Reads and writes Fountain. |
| `store/` | The only block that reads or writes a project folder (`FORMAT.md`); also the built-in object library, camera presets and lamps (data files). |
| `export/` | Turns the display list and the shot list into PDF, SVG, PNG, CSV, ODS and XLSX, and a view of the set (`stellprobe`'s picture or vector faces) into PNG or PDF pages, as bytes; colours from Kante's `palette.json` (copied by the same script). |
| `model/` | The project's data, shared by all: project, scene, character, location, floor plan, shot. |
| `imagehead/` | Reads the header of an image file; depends on nothing. |
| `demo/` (crate) | Tool, never shipped: writes the example projects into the root folder `demo/`. |
| `demo/` (root) | The demo world and the generated example projects; only those two projects ship, embedded by `services/`. |

## Layers

```
 app/ (QML, cxx-qt)      tui/ (ratatui)      cli/
    └───────────────────────┬─────────────────┘
                            ▼
 services/    project session                    demo/  tool beside services/;
                            ▼                           uses model/, script/, store/
 stellprobe/ ◄─ export/     script/     store/
                            ▼
 model/                                 imagehead/  used by store/ and export/
                                        raster/     used by stellprobe/
```

- A block depends only on the layer directly below it; the dependencies are the ones in
  its `Cargo.toml`. Nothing depends upwards.
- Exception: `services/` also depends on `model/`, whose types it passes on.
- Surfaces (`app/`, `tui/`, `cli/`) depend on `services/` only: never on files, export,
  `stellprobe/` or `model/` directly. What they need, `services/` re-exports.
- `raster/` stands beside `imagehead/`: it depends on nothing, and only `stellprobe/` uses
  it. It gets triangles in pixels and returns a frame; projection and the scene stay in
  `stellprobe/`, colouring in the surfaces.
- The middle layer stands side by side. `export/` reads the display list from
  `stellprobe/`; `store/` and `export/` read image headers through `imagehead/`; `store/`
  depends on `model/` only, not on `script/`. Otherwise they do not call each other.
- `demo/` is a tool on the level of `services/` and uses `model/`, `script/`, `store/`.
  No other block depends on it.
- Inside `app/`, QML talks only to the bridge, and the bridge only to `services/`.
- Only `store/` touches a project's files. `export/` returns bytes; `services/` has
  `store/` write them.
- Surfaces draw the display list. Geometry, hit tests and calculations belong in
  `stellprobe/`, never in QML or a surface.
- What an element looks like (colour role, line, fill) is decided once; screen and
  printout only translate it into their format, so the printout matches the screen.
- The names of a list of choices (poses, light kinds, layers …) stand once, beside the
  type; surfaces get the list from there and keep none of their own.
- Colours are roles in `model/` and `stellprobe/`, never values; `app/` maps them to
  Kante, `export/` to the Kante palette.
- Texts shown to the user come from the catalogue in `services/i18n`, German and English,
  never from code or QML.

## Where new things go

A change that needs more places than listed is a finding for the audit.

| Change | Touch |
|---|---|
| New kind of element on the plan | `model/` (data) and `FORMAT.md`; `stellprobe/` display list (role, source, shape, hit test) and its look (`look.rs`); `services/` plan edits (add, move, remove, check) and its layer; `app/` tool (`tools.rs`, `selection.rs` for what keys and drags do to the selection, `PlanTools.qml`) and the translation of the look to Kante (`plan.rs`); its tooltip (`Hint` and a `*_hint` in `services/` views, `bridge/hints.rs`, `tipLines` in `PlanArea.qml`); `export/` translation of the look to print (`style.rs`); texts. |
| New choice in a list (pose, light kind, line style, layer) | The type with its full list and names: in `model/`, layers in `services/` (`session.rs`, `session/layers.rs`); `stellprobe/` if it draws differently; texts. Nothing in a surface. |
| New field on a card (mark, camera, shot, object) | `model/` and `FORMAT.md`; the edit in `services/` (`plan_edit/marks.rs`, `camera_edit.rs`, `shot_edit.rs`, `field.rs` for refused input); the card in `app/` (`mark.rs`, `camera.rs`, `object.rs`, the card's QML, its `bridge/` file); texts; `export/` if the shot list prints it. |
| New value shown in the window | A view in `services/` (`views.rs`, `session/views.rs`); one property in the `app/` bridge (`bridge.rs`: declaration; `bridge/state.rs`: field; `bridge/start.rs`: start value) and its field in `Shown` with a line in `collect`, `fill` and `show` (`bridge/view.rs`); the QML page. |
| New export format | Writer in `export/`; one call in `services/` (`session/export.rs`); format option in `cli/`; menu entry in `app/` (the invokable in `bridge.rs`, `bridge/export.rs`, `AppHeader.qml`); texts. |
| New screenplay element | `script/` (element, parse, write, tests); the script lines in `services/` (`session/timeline.rs`); its look in `tui/` (`fountain.rs`) and in `app/` (`ScriptPanel.qml`). |
| New text in a surface | The same key in `services/i18n/de.json` and `en.json`; used through `texts.t("key")` in QML or `Language::text` in Rust. |
| New file or changed layout of the project folder | `model/` (data, `FORMAT_VERSION`); `store/`; `FORMAT.md`; `testdata/`; regenerate the examples (`cargo run -p klappe-demo`). |
| New library object, camera preset or lamp | Data only: `store/library/standard.toml`, `store/camera/standard.toml`, `store/light/standard.toml` (and its name `lightPreset.<id>` in the catalogue). |
| New template (set-up of figures and cameras) | Data only: `store/template/standard.toml` and its name `template.<id>` in the catalogue. |
| New component or colour for the window | Kante first (its own repository), then `scripts/sync-kante.sh`; never in `app/qml/Kante`. |

## Known deviations

- The window has no screenplay editor yet (`ScriptPanel.qml` only lists the script); the
  terminal writes it today. Planned in `planung/drehbuch/LASTENHEFT.md` (DB-23, after F-05).

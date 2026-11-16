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
| `script/` | Reads and writes Fountain, and the rules on its text that every editor shares: completion, the Enter and Tab flow, search and replace, the title page fields, inline marks. |
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
- `app/src/bridge.rs` holds only the declarations of the one QML object (properties, calls,
  signals); cxx-qt needs them in one module, so the file grows with the window and has no
  length limit (`structure.baseline` excludes it). Logic goes into `bridge/<area>.rs`.
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
| New kind of element on the plan | `model/` (data) and `FORMAT.md`; `store/` its check on load (a `check_*` in `lib.rs`, refused values do not load); `stellprobe/` display list (role, source, shape, hit test) and its look (`look.rs`); `services/` plan edits (add, move, remove, copy in `plan_edit/group.rs`, check) and its layer; `app/` its key and kind (`plan/key.rs`), tool (`tools.rs`, `selection.rs` for what keys and drags do to the selection, `PlanTools.qml`) and the translation of the look to Kante (`plan.rs`); its tooltip (`Hint` and a `*_hint` in `services/` views, `bridge/hints.rs`, `tipLines` in `PlanArea.qml`); `export/` translation of the look to print (`style.rs`); texts. |
| New grip on an element (drag a size, a point, an end) | `stellprobe/` `display/handles.rs` (where it sits; `Role::Handle` is the look of every grip) and its `Source` variant with the element it belongs to in `Source::owner` (`display.rs`: layer and selection read it); `services/` what dragging it changes (`plan_edit/target.rs`, `moving.rs`), `remove.rs` only if Delete does something; `plan_edit/picking.rs` `owner_of` only if the grip must catch inside its element too (it follows `Source::owner`; door and lamp grips are its two named exceptions); `app/` its key and kind (`plan/key.rs`), its `ElementKind` arm in `bridge/hints.rs` (no tooltip: `None`) and, where a click should select the grip itself, `Source::selected_itself` (`display.rs`; `selection.rs` and `bridge/view.rs` read it). Not in `export/`: grips are never printed. |
| New choice in a list (pose, light kind, line style, layer) | The type with its full list and names: in `model/`, layers in `services/` (`session.rs`, `session/layers.rs`); `stellprobe/` if it draws differently; texts. Nothing in a surface. |
| New field on a card (mark, camera, shot, object) | `model/` and `FORMAT.md`; the edit in `services/` (`plan_edit/marks.rs`, `camera_edit.rs`, `shot_edit.rs`, `field.rs` for refused input); the card in `app/` (`mark.rs`, `camera.rs`, `object.rs`, the card's QML, its `bridge/` file); texts; `export/` if the shot list prints it. |
| New key or palette command for the selection (arms the next click, acts on a way) | `app/` its edit as a function of the selection (`arm.rs`, `way.rs`, `mark.rs`), the call in `bridge/arm.rs` (declaration in `bridge.rs`, the armed state in `bridge/state.rs`, `start.rs`, `view.rs`), the key and palette entry as a `CommandAction` in `AppHeader.qml`, Escape in `PlanArea.qml`; the edit itself in `services/` (`plan_edit/moving/ways.rs` for ways of marks); texts. |
| New value shown in the window | A view in `services/` (`views.rs`, `session/views.rs`); one property in the `app/` bridge (`bridge.rs`: declaration; `bridge/state.rs`: field; `bridge/start.rs`: start value) and its field in `Shown` with a line in `collect`, `fill` and `show` (`bridge/view.rs`); the QML page. |
| New command on the plan (key or context menu) | The command in `services/`; its key in `PlanArea.qml`; its entry in `PlanContextMenu.qml` (visible for the kinds that take it, the key beside it); `services/` names the undo step (`commit_step`, the variant name of the edit) and `step.<Name>` in `de.json` and `en.json` gives the tooltip of Undo and Redo its words. |
| New export format | Writer in `export/`; in `services/` one call (`session/export.rs`) and its entry in `ExportFormat` (`export_format.rs`: name, whether it prints on paper, extension, the file name offered; the list reaches the surfaces in the project view) and in `Session::export`; format option in `cli/` (not for a view that has no state there); menu entry in `app/` (`AppHeader.qml`; the dialog and `bridge/export.rs` read the list, they name no format); texts (`menu.<name>Saved`, `export.pick.<name>`, `export.filter.<name>`). |
| New screenplay element | `script/` (element, parse, write, tests); the script lines in `services/` (`session/timeline.rs`); its look in `tui/` (`fountain.rs`) and in `app/` (`ScriptPanel.qml`). |
| New writing aid for the screenplay (suggestion list, element flow, search, title page field, inline mark) | `script/` the rule and its tests (`assist.rs`, `search.rs`, `title.rs`, `inline.rs`), re-exported in `services/` `lib.rs`; `tui/` the key (`app.rs`, `app/`), the drawing (`ui.rs`, `ui_overlay.rs`) and its help line; `app/` its editor later; texts. |
| New view setting of the plan (layer fade, saved view) or output built on one (pages per shot or camera) | `services/` the setting in `session/` (`fade.rs`, `layers.rs`, `saved.rs`) and what it changes in the display list or the picture (`item_fades`, `scene_fade`); a project edit in `project_edit/` (`views.rs`) with its data in `model/` and `FORMAT.md`; pages in `session/pages.rs` (they fade through `Page::fade`, `export/`); `app/` one property in the bridge (`fadesJson`), the calls in a `bridge/` file, the entry in `LayerPopover.qml`, `AppHeader.qml` (menu and palette); texts. |
| New text in a surface | The same key in `services/i18n/de.json` and `en.json`; used through `texts.t("key")` in QML or `Language::text` in Rust. |
| New file or changed layout of the project folder | `model/` (data, `FORMAT_VERSION`); `store/`; `FORMAT.md`; `testdata/`; regenerate the examples (`cargo run -p klappe-demo`). |
| New library object, camera preset or lamp | Data only: `store/library/standard.toml`, `store/camera/standard.toml`, `store/light/standard.toml` (and its name `lightPreset.<id>` in the catalogue). |
| New template (set-up of figures and cameras) | Data only: `store/template/standard.toml` and its name `template.<id>` in the catalogue. |
| New action on a start-page project tile (menu entry) | `services/` `folder.rs` (the folder work, through `store/` `folder.rs`; zip bytes from `export/`); `app/` one call in `bridge.rs` with its body in `bridge/project.rs`, the entry in `StartPage.qml` and its question in `TileDialog.qml`; texts `start.*`. |
| New component or colour for the window | Kante first (its own repository), then `scripts/sync-kante.sh`; never in `app/qml/Kante`. |

## Known deviations

- The window has no screenplay editor yet (`ScriptPanel.qml` only lists the script); the
  terminal writes it today. Planned in `planung/drehbuch/LASTENHEFT.md` (DB-23, after F-05).

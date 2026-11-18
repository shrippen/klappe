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
| `script/` | Reads and writes Fountain, the rules on its text that every editor shares (completion, the Enter and Tab flow, search and replace, the title page fields, inline marks, the outline of sections, scenes and synopses with moving blocks), lays the screenplay out on pages (`pages/`: Courier 12, A4 or Letter, the break rules, scene lengths in eighths), and searches script text: matches the scenes of two script states (`align`), finds quoted lines in a scene (`find_quote`), detects Git conflict markers; knows the paper only as rows per page (A4, Letter), no file or output format. |
| `store/` | The only block that reads or writes a project folder (`FORMAT.md`), the register `szenen.toml` and its Git-conflict check included; also the built-in object library, camera presets and lamps (data files). |
| `export/` | Turns the display list and the shot list into PDF, SVG, PNG, CSV, ODS and XLSX, a view of the set (`stellprobe`'s picture or vector faces) into PNG or PDF pages, a screenplay laid out by the caller (rows and columns of text, `script.rs`) into a Courier PDF, and a folder's files into a ZIP archive, as bytes (the exception: `video.rs` feeds frames to an installed `ffmpeg` and writes the MP4 itself); colours from Kante's `palette.json` (copied by the same script). |
| `model/` | The project's data, shared by all: project, the register of what is shot (a scene or a content item, with random ids), character, location, floor plan, shot. |
| `imagehead/` | Reads the header of an image file; depends on nothing. |
| `demo/` (crate) | Tool, never shipped: writes the example projects into the root folder `demo/`. |
| `demo/` (root) | The demo world and the generated example projects; only those two projects ship, embedded by `services/`. |

## Dependencies outside the workspace

- Qt Quick, Kirigami and Kante for the window (see `app/`).
- **Qt TextToSpeech** (`qt6-speech`, QML module `QtTextToSpeech`): the voice that reads the
  dialogue while playing (LH-64). Only `app/qml/SpeechVoice.qml` uses it, and it makes the
  module from text at run time, so the build needs nothing of it and a system without it
  starts with the switch off. No Rust crate depends on it; `services/` plans the speech
  without Qt.

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
  `store/` write them. One exception: the MP4 is written by ffmpeg, an external program;
  `export/` (`video.rs`) only drives it.
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
| Change to how the register follows the book (alignment, order, new entries, open questions, the answer) | `script/` `align` only for the text search itself; `services/` `book.rs` (the plan: what becomes of each entry, how it is written into the register) and `session/book.rs` (when it runs: open, save, undo; `resolve_scene`; the lines of the question dialog and the answer by position: `book_answers`, `answer_book`); `app/` and `tui/` the status line (`book_report_text`) and the question dialogs (`BookDialog.qml`, `tui/src/app/book.rs`), which only show the lines and hand back the position; `FORMAT.md` if something stored changes; texts. The same for the locations of the scene headings that no play location (`model/` `Place`, `orte.toml` in `store/` `places.rs`) is called by: `services/` `places.rs` (which are open, what each answer makes of the places) and `session/places.rs` (`place_answers`, `answer_place`; the answer is written to `orte.toml` at once, from the file on disk), `script/` `heading_place` (the location of a heading); the question dialog of the `tui/` shows them after the scene questions. |
| New kind of element on the plan | `model/` (data) and `FORMAT.md`; `store/` its check on load (a `check_*` in `lib.rs`, refused values do not load); `stellprobe/` display list (role, source, shape, hit test) and its look (`look.rs`); `services/` plan edits (add, move, remove, copy in `plan_edit/group.rs`, check; the copy of an element that is a project character too goes through `project_edit/copy.rs`) and its layer; `app/` its key and kind (`plan/key.rs`), tool (`tools.rs`, `selection.rs` for what keys and drags do to the selection, `PlanTools.qml`) and the translation of the look to Kante (`plan.rs`); its tooltip (`Hint` and a `*_hint` in `services/` views, `bridge/hints.rs`, `tipLines` in `PlanArea.qml`); `export/` translation of the look to print (`style.rs`); texts. |
| New grip on an element (drag a size, a point, an end) | `stellprobe/` `display/handles.rs` (where it sits; `Role::Handle` is the look of every grip) and its `Source` variant with the element it belongs to in `Source::owner` (`display.rs`: layer and selection read it); `services/` what dragging it changes (`plan_edit/target.rs`, `moving.rs`), `remove.rs` only if Delete does something; `plan_edit/picking.rs` `owner_of` only if the grip must catch inside its element too (it follows `Source::owner`; door and lamp grips are its two named exceptions); `app/` its key and kind (`plan/key.rs`), its tooltip: the text key in `grip_tip` (`plan/key.rs`, the bridge sends it as `kind: "grip"`, `PlanArea.qml` only shows it) and its `ElementKind` arm in `bridge/hints.rs` (a grip goes to `grip_hint`), and, where a click should select the grip itself, `Source::selected_itself` (`display.rs`; `selection.rs` and `bridge/view.rs` read it; the same grips are what a box or lasso over the selected element takes, `plan_edit/picking.rs` `point_grips`). Not in `export/`: grips are never printed. |
| New choice in a list (pose, light kind, line style, layer) | The type with its full list and names: in `model/`, layers in `services/` (`session.rs`, `session/layers.rs`); `stellprobe/` if it draws differently; texts. Nothing in a surface. |
| New way of an element through the scene (marks over time, like a figure's) | `model/` (a track in `FloorPlan`) and `FORMAT.md`; `stellprobe/` its place at `t` through the figure code (`motion.rs`, `prop.rs`: `object_at`, `set_at`; a figure's fade in and out is `motion/presence.rs`, carried to the surfaces by `Item::opacity`, `Picture::opacities` and `VectorFace::opacity`), its marks and way in the display list (`display/objects.rs`) and a `Source` variant for its mark; `services/` its edits (`plan_edit/props.rs`; `target.rs` for move and turn, `remove.rs`, `marks.rs` for the pause) and the layer of the source (`layer_of_source`, `session/layers.rs`); `app/` its key (`plan/key.rs`), the click that arms it (`arm.rs`) and the card fields (`mark.rs`, `bridge/marks.rs`, `MarkCard.qml`); texts. |
| New field on a card (mark, camera, shot, object) | `model/` and `FORMAT.md`; the edit in `services/` (`plan_edit/marks.rs`, `camera_edit.rs`, `shot_edit.rs`, `field.rs` for refused input); the card in `app/` (`mark.rs`, `camera.rs`, `object.rs`, the card's QML, its `bridge/` file); texts; `export/` if the shot list prints it. A shot's picture is a file of `bilder/` (`session/shots.rs`, via `store::add_image`; a still of its camera view via `add_image_bytes`); a project-wide choice such as the numbering scheme is a `ProjectEdit` (`project_edit.rs`) with its control in `ProjectDialog.qml`. |
| New key or palette command for the selection (arms the next click, acts on a way) | `app/` its edit as a function of the selection (`arm.rs`, `way.rs`, `mark.rs`), the call in `bridge/arm.rs` (declaration in `bridge.rs`, the armed state in `bridge/state.rs`, `start.rs`, `view.rs`), the key and palette entry as a `CommandAction` in `ArmCommands.qml` (objects: `ObjectCommands.qml`; keys that move the 3D view or the timeline's selection: `NavCommands.qml`), listed in `AppHeader.qml` `entries`, Escape in `PlanArea.qml`; the edit itself in `services/` (`plan_edit/moving/ways.rs` for ways of marks); texts. |
| Change to the scene numbers (automatic or locked, first number, A-numbers, struck scenes) | `script/` `numbering.rs` (the text, no setting known); `model/` `ScriptSetup` and `FORMAT.md` for the setting; `store/` `lock.rs` (the Sperrstand `nummern.toml`); `services/` `session/numbering.rs` (runs first in `save` and `save_script`, never on open; writes the Sperrstand last) and `book/lock.rs` (`Plan::holding`, `Plan::numbers`: who holds a number several scenes carry, what the Sperrstand becomes); `tui/` the form (`app/numbers.rs`); texts. |
| New value shown in the window | A view in `services/` (`views.rs`, `session/views.rs`); one property in the `app/` bridge (`bridge.rs`: declaration; `bridge/state.rs`: field; `bridge/start.rs`: start value) and its field in `Shown` with a line in `collect`, `fill` and `show` (`bridge/view.rs`); the QML page. |
| New command on the plan (key or context menu) | The command in `services/`; its key in `PlanArea.qml`; its entry in `PlanContextMenu.qml` (visible for the kinds that take it, the key beside it); `services/` names the undo step (`commit_step`, the variant name of the edit) and `step.<Name>` in `de.json` and `en.json` gives the tooltip of Undo and Redo its words. |
| New export format | Writer in `export/`; in `services/` one call (`session/export.rs`) and its entry in `ExportFormat` (`export_format.rs`: name, whether it prints on paper, extension, the file name offered; the list reaches the surfaces in the project view) and in `Session::export`; format option in `cli/` (not for a view that has no state there); a format written by an external program (the MP4, `video.rs`) keeps that call in `export/` and its collecting of frames in `services/` `session/video.rs` (`VideoJob`, built on the session, run on a thread of the surface: `app/` `bridge/export.rs` `write_video`); menu and palette entry in `app/` (the command in `ExportCommands.qml`, page outputs in `PageExports.qml`, the menu item in `AppHeader.qml`; the dialog and `bridge/export.rs` read the list, they name no format); texts (`menu.<name>Saved`, `export.pick.<name>`, `export.filter.<name>`). |
| New screenplay element | `script/` (element, parse, write, its line kind for quotes and prints in `quote.rs` (`LineKind`, `kind_text`), its line kind on the page `PageKind` in `pages.rs` and its place in `pages/block.rs`, tests); in `services/` the script lines (`session/timeline.rs`) and its face in the PDF (`script_pages.rs`); its look in `tui/` (line kind in `fountain.rs`, style in `ui.rs` `style_of`) and in `app/` (`ScriptPanel.qml`). |
| New writing aid for the screenplay (suggestion list, element flow, search, title page field, inline mark, outline) | `script/` the rule and its tests (`assist.rs`, `flow.rs` for the element flow, `search.rs`, `title.rs`, `inline.rs`, `outline.rs`), re-exported in `services/` `lib.rs`; `tui/` the key (`app.rs`, `app/`), the drawing (`ui.rs`, `ui_overlay.rs`) and its help line; `app/` its editor later; texts. A title page field is also a line in `TITLE_KEYS` (`title.rs`), its label in `tui/` `i18n.rs` (`FIELD_KEYS`, by field name), its place on the sheet in `services/` `script_pages.rs` and the PDF in `export/` `script.rs` (`TitlePage`; `export/` does not see `script/`). |
| New view setting of the plan (layer fade, saved view) or output built on one (pages per shot or camera) | `services/` the setting in `session/` (`fade.rs`, `layers.rs`, `saved.rs`) and what it changes in the display list or the picture (`item_fades`, `scene_fade`); a project edit in `project_edit/` (`views.rs`) with its data in `model/` and `FORMAT.md`; pages in `session/pages.rs` (they fade through `Page::fade`, `export/`); `app/` one property in the bridge (`fadesJson`), the calls in a `bridge/` file, the entry in `LayerPopover.qml`, a palette entry in `ViewCommands.qml` (`palette()`, run by a `case` in `AppHeader.qml` `run`; a plain command as a `CommandAction` there, shot views in `ShotViewCommands.qml`) and its menu item in `AppHeader.qml`; texts. |
| New thing the sun or the compass does to the plan (a place, a date, what daylight draws) | `model/` `site.rs` (`Site` of a location, `SunTime` of a scene) and `FORMAT.md`; `store/` `sun.rs` (refused values do not load); `stellprobe/` `sun.rs` (position from place and time) and `daylight.rs` (patches and arrows as items, added to the list by `add_sun`) with a `Role` and its look (`look.rs`); `services/` the edit (`EditLocation`, `EditScene` in `project_edit.rs`), the text of the forms (`sun_input.rs`), the call of `add_sun` in `session/pick.rs` and the layer of the role (`session/layers.rs`: the Lights layer); `app/` the fields of the location and scene forms (`LocationEditor.qml`, `SceneEditor.qml`, their calls in `bridge/script.rs`); `export/` `style.rs`; texts. |
| New thing read or played to the ear (speech, later sound) | `services/` what plays when, as a plan without Qt (`session/speech.rs`, `Session::speech`); `app/` one call in the bridge (`speech_json`), the voice as a QML object (`SpeechVoice.qml`, Qt TextToSpeech) and the player that follows the playhead (`SpeechPlayer.qml`); the switch in `Timeline.qml`, the palette entry in `PlayCommands.qml`, its choice in the window's settings (`Main.qml`); texts. |
| Jump between a script line and what it is linked to (DB-63) | `services/` the link data (`Chip` in `views.rs`, built in `session/timeline.rs`); `app/` the calls in `bridge/jump.rs` (declaration in `bridge.rs`), the commands as `CommandAction` in `JumpCommands.qml` (listed in `AppHeader.qml` `entries` and the View menu), the showing in `Workspace.qml` (`revealScript`, `revealPlan`) and `ScriptPanel.qml` (`showLine`, `showHead`, chips); texts `script.*`. The terminal has no jump. |
| New text in a surface | The same key in `services/i18n/de.json` and `en.json`; used through `texts.t("key")` in QML or `Language::text` in Rust. |
| New file or changed layout of the project folder | `model/` (data, `FORMAT_VERSION`); `store/`; `FORMAT.md`; `testdata/`; regenerate the examples (`cargo run -p klappe-demo`). |
| New library object, camera preset or lamp | Data only: `store/library/standard.toml` (and its 3D form in `forms.toml`, its name `objectKind.<kind>` and group `objectGroup.<group>` in the catalogue), `store/camera/standard.toml`, `store/light/standard.toml` (and its name `lightPreset.<id>` in the catalogue). |
| New template (set-up of figures and cameras) | Data only: `store/template/standard.toml` and its name `template.<id>` in the catalogue. |
| New action on a start-page project tile (menu entry) | `services/` `folder.rs` (the folder work, through `store/` `folder.rs`; zip bytes from `export/`); `app/` one call in `bridge.rs` with its body in `bridge/project.rs`, the entry in `StartPage.qml` and its question in `TileDialog.qml`; texts `start.*`. |
| New component or colour for the window | Kante first (its own repository), then `scripts/sync-kante.sh`; never in `app/qml/Kante`. |

## Known deviations

- The window has no screenplay editor yet (`ScriptPanel.qml` only lists the script); the
  terminal writes it today. Planned in `planung/drehbuch/LASTENHEFT.md` (DB-23, after F-05).

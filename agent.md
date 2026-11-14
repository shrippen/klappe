# Working rules for this repository

The rules for all shrippen repositories are in `RULES.md` (writing, code, commits, bug
fixes, repository, GUI). It is a synced copy from the rules repository; do not edit it
here. This file holds only what is particular to this repository. A Claude Code session on
the owner's machine has the rules through the global `CLAUDE.md` already and reads
`RULES.md` only for its GUI rule, before work on a user interface.

## Klappe – Projekt-Wissen

## Zweck
Film-Vorproduktionssuite (Linux zuerst, dann Windows, macOS). Erstes Modul **Stellprobe**:
Floorplans mit Blocking, Kameras, Licht, 180°-Achse, Shotliste. Später Drehbuch (Fountain,
TUI) und Planung. Anforderungen: `planung/stellprobe/LASTENHEFT.md` (Clean Room, Anhang A);
Technik und Entscheidungen: `planung/KONZEPT.md`; Arbeitsstand: `planung/STAND.md`.

Implementierung: Rust (Cargo-Workspace), GUI in QML (Kirigami + Kante) über cxx-qt, TUI mit
ratatui, Lizenz MIT.

## Schichten (nur zum direkten Nachbarn darunter)
```
app/ (QML, cxx-qt)    tui/ (ratatui)    cli/
        └──────────────┬─────────────────┘
                       ↓
services/   Projekt-Sitzung: öffnen, speichern, Rückgängig, Wiedergabeuhr
                       ↓
stellprobe/ Optik, Bewegung, Achse, Darstellungsliste    script/ Fountain
store/      Projektordner                                export/ PDF, SVG, CSV
                       ↓
model/      Projekt, Szene, Figur, Drehort, Shot (für alle Module)
imagehead/  JPEG-Kopf lesen (ohne Abhängigkeit; store/ und export/ rufen es)

demo/       Werkzeug (crates/demo), nicht ausgeliefert: erzeugt die Beispielprojekte im
            Ordner demo/ des Repos über model/, script/, store/ (neben services/)
```
- Wie `planung/KONZEPT.md`, Abschnitt 4. Die mittlere Ebene steht nebeneinander: `export/`
  liest die Darstellungsliste aus `stellprobe/`, `store/` nutzt `script/`; beide lesen JPEG-Köpfe über
  `imagehead/`; sonst rufen sie sich nicht.
- Oberflächen zeichnen die Darstellungsliste aus `stellprobe/`; eigene Geometrie in QML ist ein
  Fehler.
- Jede Schicht ist ein eigenes Crate; Abhängigkeiten nur zum Nachbarn darunter (in
  `Cargo.toml` sichtbar). `demo/` ist ein Werkzeug auf der Ebene von `services/`; App, TUI und
  CLI hängen nie an ihm, ausgeliefert wird nur das erzeugte Beispiel (über `services/`).
- Alles privat; `pub` nur, wo das Design es braucht. Jede neue `pub`-Schnittstelle steht im PR.

## Konventionen
- Rust (stable), cargo fmt, clippy ohne Warnungen. `make check` vor jedem Commit.
- Kein `unsafe` außerhalb der cxx-qt-Brücke; jedes `unsafe` mit Kommentar, warum es sicher ist.
- Kein `unwrap()`/`expect()` außerhalb von Tests; Fehler als eigene Fehlertypen je Crate.
- Abhängigkeiten nur unter MIT, Apache-2.0, BSD, ISC, Zlib, Unlicense, CC0 (Schriften:
  SIL OFL); Qt und KDE Frameworks (LGPL) nur dynamisch. Keine GPL, MPL oder LGPL-Crates
  (`deny.toml`). Neue Abhängigkeit nur mit Begründung im PR; im Zweifel selbst bauen.
- Einheiten im Namen: Meter `_m`, Grad `_deg`, Sekunden `_s`, Millimeter `_mm`, Kelvin `_k`.
  Plan-Koordinaten: Ursprung oben links, y nach unten, 0° nach rechts, im Uhrzeigersinn.
- Farben im Modell sind Rollen (`FigureColor`, `display::Role`), nie Hex-Werte; die GUI
  bildet sie auf Kante ab.
- Texte der GUI deutsch und englisch, nur über Übersetzungsschlüssel.
- Clean Room: Shot Designer nie disassemblieren, sein Dateiformat nie auslesen, keine
  Grafiken oder Texte übernehmen (Lastenheft, Anhang A).
- Temporäre Dateien und `CARGO_TARGET_DIR` unter `~/.cache/claude/klappe-*`, nicht `/tmp`.

## Gelernte Fehler

- Prüfkette vor einem Commit mit `&&`, nicht mit `;`: sonst wird trotz rotem Build committet
  (13.11.2026, Syntaxfehler im Commit).
- Qt schreibt Meldungen (auch `console.log`) ins Journal, wenn stderr kein Terminal ist: in
  Tests und Skripten `QT_FORCE_STDERR_LOGGING=1` setzen, sonst sieht ein hängender Start leer aus.
- `grabToImage` auf dem Fenster scheitert („item has no QML engine“); Screenshots nehmen die
  Seite (`page`), wie in Kontra.

## GUI (this project)

- `crates/app/qml/Kante` is copied unchanged from Kante (`scripts/sync-kante.sh`); never edit
  it. Missing components (timeline lanes, plan canvas parts) are added to Kante first, then
  used here.
- Programs with a window run only offscreen in tests and screenshots
  (`QT_QPA_PLATFORM=offscreen`), never in a visible window.

## Repository (this project)

- The merge waits for: CI is green on the PR's latest commit: the newest status
  `github / test` there is success (`.github/workflows/ci.yml` on the GitHub mirror, every
  pushed commit; `scripts/gitea/merge-when-green.sh`). No status there yet means wait, not
  merge.
- PR-Agent: `.gitea/workflows/pr-agent.yml` as in Kontra; it also keeps Gitea from running
  `.github/workflows/` itself. Paused like everywhere (skill `pr-review`): a new Claude agent
  reviews every round.
- The owner also confirms: a PR that changes `packaging/`.
- Claude merges PRs itself when `github / test` is green and the review leaves no **Wichtig**
  open (owner, 13.11.2026: „Pushe und merge wenn Review und CI grün sind“).
- Everything else: `RULES.md` (Repository) and the skill `pr-review`.

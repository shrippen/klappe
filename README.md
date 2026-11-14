# Klappe

Film-Vorproduktionssuite für Linux, Windows und macOS. Erstes Modul: **Stellprobe**,
Floorplans mit Blocking, Kameras, Licht und Shotliste.

Stand: Phase 0 (Gerüst). Arbeitsstand: [`planung/STAND.md`](planung/STAND.md).
Bauen und prüfen: `make check` (Rust stable, `cargo-deny`, `pipx`, Qt 6 mit Kirigami).
Starten mit dem Beispielprojekt: `make run`.

Drehbuch schreiben im Terminal: `cargo run -p klappe-tui -- testdata/beispiel.klappe` (F1 zeigt die Tasten,
`--language de|en` wählt die Sprache).

Konzept: [`planung/KONZEPT.md`](planung/KONZEPT.md), Oberfläche: [`planung/DESIGN.md`](planung/DESIGN.md),
Lastenheft Stellprobe: [`planung/stellprobe/LASTENHEFT.md`](planung/stellprobe/LASTENHEFT.md).

Lizenz: MIT.

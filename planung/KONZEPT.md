# Konzept: Klappe

Film-Vorproduktionssuite. Stand: 2026-11-13. Entscheidungen: Abschnitt 7.
Anforderungen des ersten Moduls: [`stellprobe/LASTENHEFT.md`](stellprobe/LASTENHEFT.md).

## 1. Module

| Modul | Zweck | Oberfläche | Reihenfolge |
|---|---|---|---|
| Stellprobe | Floorplans, Blocking, Kameras, Licht, Shotliste | GUI | zuerst |
| Drehbuch | Drehbuch schreiben (Fountain) | **TUI**, später GUI | zweites |
| Planung | Drehplan, Dispo, Drehorte, Team | GUI | später |

Alle Module arbeiten auf demselben Projekt und demselben Datenmodell.

## 2. Vorbild: eigene Projekte

| Projekt | Stack | Übernommen wird |
|---|---|---|
| Kontra | Rust-Workspace, QML über cxx-qt, Kirigami, Kante, Flatpak, MIT | Stack, Schichten als Crates, Abhängigkeitsregel, `make check`, `planung/` |
| Mitori | C++, Qt 6, Kirigami, Android | Plattformschicht, Android-Erfahrung (falls Tablet Muss wird) |
| Plasmai | QML, App mit eigener TUI, Windows-Client | Ein Kern, mehrere Oberflächen; Windows-Paket |

## 3. Technische Basis (entschieden: Variante A)

### Variante A: Rust-Kern, Qt/QML-GUI, ratatui-TUI

| Bereich | Wahl | Lizenz | Begründung |
|---|---|---|---|
| Sprache | Rust (stable) | MIT/Apache-2.0 | Ein Kern für GUI, TUI und CLI; Erfahrung aus Kontra. |
| GUI | Qt 6, QML, Kirigami, Kante-QML | LGPL / Kante | Kante-Regel; läuft auf Linux, Windows, macOS. |
| Rust ↔ QML | cxx-qt | MIT/Apache-2.0 | Wie Kontra. |
| Plan-Zeichenfläche | eigenes `QQuickPaintedItem` bzw. Scene-Graph-Item, zeichnet die Darstellungsliste des Kerns | – | Flüssig bei NF-05; dieselbe Liste wie im Export. |
| TUI | ratatui + crossterm | MIT | Läuft in jedem Terminal, auch unter Windows. |
| Texteditor (TUI) | eigener Editor auf ropey | MIT | Fountain braucht eigenes Verhalten (Figurennamen, Szenenköpfe, Einrückung). |
| Drehbuchformat | Fountain, eigener Parser | MIT | Offener Klartext-Standard; Git-tauglich; von anderen Programmen lesbar. |
| PDF | krilla | MIT/Apache-2.0 | Headless, ohne Qt; Export auch über CLI. |
| SVG, CSV | eigene Ausgabe | MIT | klein |
| XLSX | rust_xlsxwriter | MIT/Apache-2.0 | LH-85 |
| Kamerablick (3D) | eigene einfache Perspektivprojektion im Kern (Wände, Kästen, Figuren als Silhouetten) | MIT | Qt Quick 3D ist nur unter GPLv3 frei; reicht für LH-36 ohne GPU-Stack. |
| Pakete | Flatpak zuerst, dann Windows, dann macOS | – | Wie Kontra. |

Abhängigkeitsregel wie Kontra: nur MIT, Apache-2.0, BSD, ISC, Zlib, Unlicense, CC0, Schriften
SIL OFL; Qt und KDE Frameworks dynamisch; `cargo-deny` prüft. Im Zweifel selbst bauen.

**Risiko:** cxx-qt auf Android ist wenig erprobt. Kommt später ein Tablet-Build, wird das zuerst geprüft.

### Variante B: C++ und Qt 6 (wie Mitori), verworfen

GUI und Kern in C++, TUI mit FTXUI (MIT) oder eigenem Terminal-Code wie Plasmai.
Vorteil: Android erprobt, Qt-Items ohne Brücke. Nachteil: kein gemeinsamer Rust-Bestand mit
Kontra und lmix, weniger Sicherheit, TUI und Export hängen an Qt oder doppeltem Code.

### Verworfen

- Rust mit egui, iced oder Slint: Kante gibt es nur für QML.
- Electron/Web: kein natives Programm, schwer, widerspricht dem Ziel.

## 4. Schichten (nur zum direkten Nachbarn darunter)

```
app/   QML, Kirigami, Kante      tui/  ratatui        cli/  Kommandozeile
   │ (cxx-qt-Brücke)                │                    │
   └────────────────┬───────────────┴────────────────────┘
                    ↓
services/   Projekt-Sitzung: öffnen, speichern, Autosave, Rückgängig, Wiedergabeuhr
                    ↓
stellprobe/ Optik, Bewegung, Achse, Darstellungsliste   script/  Fountain
store/      Projektordner lesen und schreiben           export/  PDF, SVG, CSV, XLSX
                    ↓
model/      Projekt, Szene, Figur, Drehort, Shot (für alle Module)
```

- Jede Schicht ein Crate unter `crates/`; Abhängigkeiten sichtbar in `Cargo.toml`.
- Oberflächen rufen nur `services/`; nie Dateien, Export oder Kern direkt.
- Die mittlere Ebene steht nebeneinander: `export/` liest die Darstellungsliste aus
  `stellprobe/`, `store/` nutzt `script/` für das Drehbuch. Sonst rufen sie sich nicht.
- `stellprobe` erzeugt eine **Darstellungsliste** (Linien, Flächen, Kreise, Beschriftungen in
  Metern, Farben als Rollen). Die GUI zeichnet sie, `export/` schreibt sie als PDF oder SVG.
  Was am Bildschirm steht, steht genauso im Ausdruck, und Tests prüfen die Liste ohne Fenster.

## 5. Projektformat

Ein Projekt ist ein Ordner aus Textdateien, damit Git es versionieren und vergleichen kann (NF-03):

```
wartburg.klappe/
├── projekt.toml            Formatversion, Titel
├── figuren.toml            Figuren: Id, Name, Kürzel, Farbe
├── drehbuch.fountain       Drehbuch; Szenen tragen feste Nummern (#12#)
├── drehorte/
│   └── saengersaal.toml    Drehort mit Set: Maße, Wände, Objekte
└── szenen/
    └── szene-012.json      Szene mit ihren Floorplans: Marks, Kameras, Licht, Shots, Verweise
```

Genaue Beschreibung: [`../FORMAT.md`](../FORMAT.md). Hintergrundbilder (LH-13) und eigene
Bibliotheken (LH-12) kommen später dazu.

- Feste Ids als Dateinamen; JSON eingerückt mit fester Schlüsselreihenfolge, damit Diffs
  lesbar bleiben.
- Ein JSON-Schema in `schema/` folgt später (wie Kontra).
- Zum Weitergeben: derselbe Ordner als Zip (`.klappe`-Datei).

## 6. Phase 0: Prototyp, bevor gebaut wird

1. cxx-qt + Kirigami + Kante: Zeichenfläche mit 300 Objekten, 50 Figuren, 10 Kameras,
   flüssiges Verschieben und Zoomen (NF-05).
2. Darstellungsliste → Bildschirm und PDF sehen gleich aus.
3. TUI-Editor: Fountain tippen mit automatischer Formatierung von Szenenkopf, Figur, Dialog.
4. Flatpak und Windows-Build in der CI.

## 7. Entscheidungen

Eigentümer, 2026-11-13:

| Nr | Frage | Entscheidung |
|---|---|---|
| E-01 | Tablet am Set | Nein; erste Version nur Desktop und Laptop (NF-07 bleibt Kann). |
| E-02 | Shot-Designer-Pläne übernehmen | Nein; bei Bedarf von Hand nachbauen. |
| E-03 | Kamerablick (LH-36) | Soll: nach den Muss-Anforderungen, vor 1.0. |
| E-04 | Weitergabe an Kamera, Licht, Ausstattung | PDF reicht. |
| E-05 | Lizenz | MIT; Abhängigkeitsregel wie Kontra. |
| E-06 | Technische Basis | Variante A: Rust-Kern, Qt/QML über cxx-qt, TUI mit ratatui. |
| E-07 | Plattformen | Linux (Flatpak), dann Windows, dann macOS. |
| E-08 | Drehbuchformat | Fountain. |
| E-09 | Umfang der TUI | Nur Drehbuch schreiben. |
| E-10 | Programme | GUI-App `klappe` mit Modulen, dazu `klappe-tui` und `klappe-cli`. |
| E-11 | Projektformat | Ordner aus Textdateien; zum Weitergeben als Zip (`.klappe`). |
| E-12 | Technik des Kamerablicks | Eigene einfache Perspektivprojektion im Kern. |
| E-13 | CI | Wie Kontra: Prüfungen und Flatpak auf dem GitHub-Mirror, Status an Gitea. |
| E-14 | Oberfläche | Entwurf B der Runde 2 („Drehbuch daneben“): Drehbuch links, Plan in der Mitte, Shot mit Vorschau und Daten rechts, Zeitleiste mit Ebenenschaltern unten. Seitenleisten und Zeitleiste ein- und ausklappbar; oben Kopfzeile und Werkzeugleiste wie Entwurf A. Entwürfe: https://claude.ai/artifact/9UDTh7hdqccSMz62NLZztN |

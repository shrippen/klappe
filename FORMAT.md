# Projektformat

Ein Klappe-Projekt ist ein Ordner aus Textdateien, damit Git es versionieren und vergleichen
kann (NF-03). Umsetzung: `crates/store`; Beispiel: `testdata/beispiel.klappe/`.

```
<name>.klappe/
├── projekt.toml         Formatversion, Titel
├── figuren.toml         Figuren, je ein [[figur]]
├── drehorte/<id>.toml   ein Drehort samt Set (Wände, Objekte)
├── szenen/<id>.json     eine Szene: Nummer, Überschrift, Drehort, Floorplans
├── drehbuch.fountain    Drehbuch als Text (optional)
└── .sicherung/          Sicherung ungespeicherter Änderungen (optional, siehe unten)
```

## Regeln

- **Formatversion:** `format_version` in `projekt.toml` (aktuell 1). Eine neuere Version als die
  des Programms wird nicht geladen, sondern gemeldet.
- **Ids:** Drehort- und Szenen-Ids sind Dateinamen und müssen mit der `id` in der Datei
  übereinstimmen. Nicht erlaubt: leer, `/`, `\`, `..`, Beginn mit `.`, Ende auf `.` oder Leerzeichen,
  `<>:"|?*`, Steuerzeichen, Windows-Gerätenamen (`con`, `nul`, `com1` …). Ids, die sich nur
  in der Groß-/Kleinschreibung unterscheiden, gelten als gleich.
- **Reihenfolge:** Drehorte und Szenen werden nach Id sortiert gelesen. Die Reihenfolge der
  Figuren steht in `figuren.toml`, die der Shots (Schnittfolge) im Floorplan.
- **Shots:** `size` (Einstellungsgröße), `angle` (Perspektive) und `movement` sind Namen in
  Kleinschreibung mit Bindestrich. `angle`: `eye-level` (Normalsicht), `high` (Aufsicht), `low`
  (Untersicht), `birds-eye` (Vogelperspektive), `worms-eye` (Froschperspektive), `dutch`
  (gekippt). Fehlt `angle`, gilt `eye-level`. Die Drehreihenfolge (Setups gruppiert) steht nicht
  in der Datei; sie ergibt sich aus Kamera, Ort und Objektiv.
- **JSON:** eingerückt, Schlüssel in der Feldreihenfolge des Modells, Zeilenumbruch am Ende.
- **Einheiten:** im Feldnamen (`_m`, `_deg`, `_s`, `_mm`, `_k`, `_w`); Ursprung oben links, y nach
  unten.
- **Speichern:** jede Datei wird in eine Temp-Datei (`.<name>.tmp`) im selben Ordner geschrieben
  und dann umbenannt; ein Absturz hinterlässt nie eine halbe Datei (LH-05). Dateien in
  `drehorte/` und `szenen/`, deren Id nicht mehr zum Projekt gehört, werden gelöscht, sonst
  nichts. Ohne Drehbuchtext bleibt eine vorhandene `drehbuch.fountain` unberührt.
- Fehlende `figuren.toml`, fehlende Ordner und fehlendes Drehbuch gelten als leer.
- **Sicherung (LH-05):** ungespeicherte Änderungen schreibt die Sitzung regelmäßig nach
  `.sicherung/`, ein vollständiger Projektordner im selben Format. Die Projektdateien bleiben
  dabei unberührt; erst der Nutzer speichert, und das Speichern löscht `.sicherung/`, außer die Sitzung
  hat sie beim Öffnen gefunden und der Nutzer hat noch nicht entschieden. Beim
  Öffnen bietet die Sitzung eine abweichende Sicherung zur Wiederherstellung an; eine
  gleiche wird still gelöscht. Ein Sicherungsordner gilt nur mit
  der Datei `fertig`: sie wird zuletzt geschrieben und beim Löschen zuerst entfernt, ein
  Absturz hinterlässt also nie einen lesbaren halben Ordner. Die Sicherung wird als Ganzes
  ersetzt: erst entsteht `.sicherung.neu/` (zuletzt `fertig`), dann wird `.sicherung/` zu
  `.sicherung.alt/` umbenannt, `.sicherung.neu/` zu `.sicherung/` und `.sicherung.alt/`
  gelöscht. Ein fertiger `.sicherung.neu/` ist neuer als `.sicherung/` und wird zuerst
  gelesen; vor dem nächsten Schreiben wird er befördert, nie gelöscht. Ordner ohne `fertig`
  sind Absturzreste und werden ignoriert. Speichert der
  Nutzer nur das Drehbuch, wird die Sicherung entfernt, wenn nur das Drehbuch ungesichert
  war, sonst mit dem gespeicherten Drehbuch neu geschrieben. `load` liest `.sicherung/` nie
  als Teil des Projekts. Git soll die Ordner ignorieren; Vorschlag für `.gitignore` im Projekt:

  ```
  .sicherung/
  .sicherung.neu/
  .sicherung.alt/
  ```

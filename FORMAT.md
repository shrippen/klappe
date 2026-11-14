# Projektformat

Ein Klappe-Projekt ist ein Ordner aus Textdateien, damit Git es versionieren und vergleichen
kann (NF-03). Umsetzung: `crates/store`; Beispiel: `testdata/beispiel.klappe/`.

```
<name>.klappe/
├── projekt.toml         Formatversion, Titel
├── figuren.toml         Figuren, je ein [[figur]]
├── drehorte/<id>.toml   ein Drehort samt Set (Wände, Objekte)
├── szenen/<id>.json     eine Szene: Nummer, Überschrift, Drehort, Floorplans
├── bilder/<name>.png    Bilder für Hintergründe (PNG, JPEG), vom Nutzer hinzugefügt
└── drehbuch.fountain    Drehbuch als Text (optional)
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
- **JSON:** eingerückt, Schlüssel in der Feldreihenfolge des Modells, Zeilenumbruch am Ende.
- **Einheiten:** im Feldnamen (`_m`, `_deg`, `_s`, `_mm`, `_k`, `_w`); Ursprung oben links, y nach
  unten.
- **Speichern:** jede Datei wird in eine Temp-Datei (`.<name>.tmp`) im selben Ordner geschrieben
  und dann umbenannt; ein Absturz hinterlässt nie eine halbe Datei (LH-05). Dateien in
  `drehorte/` und `szenen/`, deren Id nicht mehr zum Projekt gehört, werden gelöscht, sonst
  nichts. Ohne Drehbuchtext bleibt eine vorhandene `drehbuch.fountain` unberührt.
- **Hintergrund:** `[set.background]` im Drehort (Datei relativ zum Projektordner, linke obere
  Ecke `at`, `width_m`, `rotation_deg` im Uhrzeigersinn um `at`, `opacity`). Die Höhe folgt aus
  dem Seitenverhältnis des Bildes. Fehlt der Abschnitt, gibt es keinen Hintergrund.
- **Bilder:** `bilder/` gehört dem Nutzer. `save` löscht dort nie etwas, auch nicht, wenn kein
  Drehort das Bild mehr nutzt. Beim Hinzufügen wird nicht doppelt kopiert (gleicher Name und
  gleicher Inhalt); anderer Inhalt unter demselben Namen bekommt `-2`, `-3` … vor der Endung.
- Fehlende `figuren.toml`, fehlende Ordner und fehlendes Drehbuch gelten als leer.

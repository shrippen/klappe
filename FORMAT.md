# Projektformat

Ein Klappe-Projekt ist ein Ordner aus Textdateien, damit Git es versionieren und vergleichen
kann (NF-03). Umsetzung: `crates/store`; Beispiel: `testdata/beispiel.klappe/`.

```
<name>.klappe/
├── projekt.toml         Formatversion, Titel
├── figuren.toml         Figuren, je ein [[figur]]
├── bibliothek.toml      eigene Objekte der Bibliothek, je ein [[objekt]] (optional)
├── drehorte/<id>.toml   ein Drehort samt Set (Wände, Objekte)
├── szenen/<id>.json     eine Szene: Nummer, Überschrift, Drehort, Floorplans
├── bilder/<name>.png    Bilder für Hintergründe (PNG, JPEG), vom Nutzer hinzugefügt
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
- **Hintergrund:** `[set.background]` im Drehort (Datei relativ zum Projektordner, linke obere
  Ecke `at`, `width_m`, `rotation_deg` im Uhrzeigersinn um `at`, `opacity`). Die Höhe folgt aus
  dem Seitenverhältnis des Bildes. Fehlt der Abschnitt, gibt es keinen Hintergrund.
- **Objekte und Bibliothek (LH-11, LH-12, LH-25):** `[[set.objects]]` im Drehort haben `kind`,
  `label`, Mitte `at`, `width_m`, `depth_m` und `rotation_deg`; dazu `shape`: `rect` oder
  `ellipse` (in `width_m` mal `depth_m` einbeschrieben). Fehlt `shape`, gilt `rect`; ein Rechteck
  schreibt den Schlüssel nicht, ältere Projekte laden und speichern unverändert. Ein `kind`,
  den keine Bibliothek kennt, bleibt ladbar und wird als Rechteck in der eigenen Größe
  gezeichnet. Die eingebaute Bibliothek ist Teil des Programms (`crates/store/library/standard.toml`,
  Namen im Katalog unter `objectKind.<kind>`). `bibliothek.toml` hält die eigenen Objekte des
  Projekts, je ein `[[objekt]]` mit `kind` (eindeutig, auch gegenüber den eingebauten), `label`
  (Name), `width_m`, `depth_m` und `shape`. Dieselbe Struktur hat eine Bibliotheksdatei zum
  Weitergeben (beliebiger Name, `*.toml`). Eine Datei mit einem unbrauchbaren Eintrag (leerer
  `kind`, Maß nicht in (0, 1000] m, doppelter `kind`) wird als Ganzes abgelehnt; die
  `bibliothek.toml` eines Projekts dann beim Öffnen mit Meldung. Ohne eigene Objekte gibt es die
  Datei nicht: `save` löscht sie dann. Keine neue Formatversion: ältere Fassungen kennen `shape`
  und `bibliothek.toml` nicht und verlieren sie beim Speichern.
- **Notizen (LH-16):** `[[set.notes]]` im Drehort: `id` (`n1`, `n2` …, innerhalb des Drehorts),
  Ort `at` in Metern und `text`. Fehlt der Abschnitt, gibt es keine Notizen; ohne Notizen wird
  er nicht geschrieben, ältere Projekte laden und speichern unverändert. Keine neue
  Formatversion: ein Feld mehr, das ältere Fassungen nicht kennen und beim Speichern verlieren.
- **Zeichnungen (LH-19):** `drawings` im Floorplan der Szene (`szenen/<id>.json`), je Eintrag
  `id` (`d1`, `d2` …, innerhalb des Floorplans), `shape`, `color`, `dash` und `fill`. `shape` hat
  `kind`: `line` und `arrow` (Spitze bei `to`) mit `from` und `to`; `rect` und `ellipse` mit zwei
  gegenüberliegenden Ecken `from` und `to` (achsenparallel); `area` mit `points` (geschlossenes
  Polygon, mindestens drei). Punkte in Metern. `color` ist eine Rolle wie bei Figuren (`purple`,
  `blue`, `green`, `yellow`, `orange`, `red`, `cyan`), `dash` `solid`, `dashed` oder `dotted`
  (Standard `solid`), `fill` `outline` oder `filled` (Standard `outline`; nur Rechteck, Ellipse und
  Fläche haben ein Inneres). Ohne Zeichnungen fehlt der Schlüssel; ältere Projekte laden und
  speichern unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Kurvenwege (LH-21):** jeder Mark einer Figur und einer Kamera (`marks` in `szenen/<id>.json`)
  hat optional `via`, einen Punkt in Metern: der Weg vom vorigen Mark zu diesem ist die
  quadratische Bézierkurve mit `via` als Kontrollpunkt (die Kurve zieht zu `via` hin, geht aber
  nicht durch). Ohne `via` ist der Weg gerade; am ersten Mark zählt `via` nicht. Die Bewegung
  läuft entlang der Kurve mit gleichmäßigem Tempo, die Ankunftszeiten bleiben die der Marks.
  Ohne Kurven fehlt der Schlüssel; ältere Projekte laden und speichern unverändert. Keine
  neue Formatversion, wie bei den Notizen.
- **Auftritt und Abgang (LH-26):** jeder Mark einer Figur (`marks` in `szenen/<id>.json`) hat
  optional `arrival` (`walk` oder `appear`, Standard `walk`) und `departure` (`stay` oder
  `leave`, Standard `stay`). Bei `appear` ist die Figur vor `at_s` dieses Marks nicht zu sehen
  und erscheint dort, statt vom vorigen Mark zu laufen; am ersten Mark heißt das: vor `at_s`
  unsichtbar. Bei `leave` ist sie nach `at_s + hold_s` nicht mehr zu sehen, bis ein späterer Mark
  `appear` hat. Ein Wert mit Standard wird nicht geschrieben; ältere Projekte laden und speichern
  unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Bilder:** `bilder/` gehört dem Nutzer. `save` löscht dort nie etwas, auch nicht, wenn kein
  Drehort das Bild mehr nutzt. Beim Hinzufügen wird nicht doppelt kopiert (gleicher Name und
  gleicher Inhalt); anderer Inhalt unter demselben Namen bekommt `-2`, `-3` … vor der Endung.
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

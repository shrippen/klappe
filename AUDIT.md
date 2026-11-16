# Code-Audit Klappe, 16.11.2026, Stand 95f7301

## Urteil
Das Repo ist wartbar und etwas besser als beim zwölften Audit. Vier der fünf Funde sind behoben (#187, #189, #191, #192), und zwar sauber:
- Möbel und Bilder rechnen jetzt mit derselben Rechnung für gedrehte Rechtecke.
- Welcher Griff zu welchem Element gehört, steht an einer Stelle.
- Stellprobe fragt die Bildgröße nur noch auf einem Weg ab.

Den fünften Fund hat der Eigentümer nicht gewählt. Neu dazugekommen sind das Menü der Projektkacheln auf der Startseite (Umbenennen, Archivieren, Papierkorb; #195) und die neue Beispielkopie (#194). Beide haben kleine Doppelungen mitgebracht. An einer Stelle geht die Sitzung dabei selbst an die Dateien, an der Ablage vorbei (Fund 1). Die Bestandsliste bleibt bei 31 Einträgen. Zwei davon sind leicht gewachsen, beide mit Begründung.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 412 | 425 (gezählt wie die Strukturprüfung, ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | 31 | 31. Angehoben hat sie nur #190, beide Male mit Begründung im Commit `a8e2cb9`: die Feldnamen `name` in `field.rs` (43 auf 44 Verzweigungen, für die neue Deckkraft) und `apply` in `plan_edit.rs` (139 auf 141 Zeilen, für die Bildänderungen). `error_text` heißt jetzt `own_text`; der Wert ist gleich geblieben. |
| größte Datei (Zeilen) | 1011 (`crates/store/tests/store.rs`) | 1011 (dieselbe). Ohne Tests ist die größte geprüfte Datei weiter `crates/export/src/jpeg.rs` mit 883 Zeilen. Die nicht geprüfte `bridge.rs` ist von 833 auf 864 Zeilen gewachsen. |
| längste Funktion (Zeilen) | 139 (`apply` in `plan_edit.rs`) | 141 (dieselbe) |

## Funde

### 1. Die Beispielkopie wird an der Ablage vorbei beiseitegelegt, mit einer eigenen Regel für den freien Namen
Lädt die Kopie des Beispiels nicht mehr, benennt die Sitzung den Ordner um und schreibt eine frische Kopie. Dieses Umbenennen macht die Sitzung selbst, direkt auf der Platte. `ARCHITECTURE.md` sagt aber: Nur die Ablage fasst die Dateien eines Projekts an. Die Ablage hat dafür eine eigene Funktion, die hier nicht benutzt wird.

Dazu kommt: Für „Welcher Name neben dem Ordner ist noch frei?“ gibt es jetzt zwei Regeln. Die Sitzung hängt `.alt`, `.alt-2` … an und sucht ohne Obergrenze. Die Ablage (für das ZIP-Archiv, #195) hängt ` (2)`, ` (3)` … an und gibt nach 1000 Versuchen auf.

Was es kostet: Wer ändert, wie Ordner auf der Platte umbenannt oder benannt werden (etwa für Windows oder einen Netzordner), muss eine Stelle außerhalb der Ablage kennen. Die beiden Namensregeln verhalten sich schon heute verschieden.

Vorschlag: Die Ablage bekommt eine Funktion „Ordner beiseitelegen“, die ihre Regel für den freien Namen benutzt. Die Sitzung ruft nur diese Funktion auf.

Aufwand klein. Risiko gering: `crates/services/tests/example.rs` (auch für den zweiten freien Namen) und `crates/services/tests/folder.rs` laufen in diesem Audit grün. Ändert sich das Namensmuster, müssen diese Tests mitgeändert werden.

Beleg: `crates/services/src/example.rs:46`, `crates/services/src/example.rs:62`, `crates/store/src/folder.rs:124`, `crates/store/src/files.rs:150`, `ARCHITECTURE.md:65`.

### 2. Die Regel für einen gültigen Projekttitel steht zweimal
Ein Titel wird beim Anlegen eines Projekts und beim neuen Umbenennen auf der Startseite (#195) gleich geprüft: Leerzeichen am Rand weg, leer ist abgelehnt. Die Regel ist dafür zweimal Zeile für Zeile ausgeschrieben.

Was es kostet: Eine neue Regel für Titel (zum Beispiel eine Höchstlänge oder verbotene Zeichen) gilt sonst nur für einen der beiden Wege.

Vorschlag: Eine Funktion „Titel prüfen“ in der Sitzung, die beide Stellen aufrufen.

Aufwand klein. Risiko gering: `crates/services/tests/folder.rs` prüft den leeren Titel beim Umbenennen. Für das Anlegen sollte vor dem Umbau ein Test auf den leeren Titel dazukommen.

Beleg: `crates/services/src/folder.rs:17`, `crates/services/src/session/file.rs:59`.

### 3. Jede Karte sucht ihr Element mit demselben kopierten Block, jetzt siebenmal
Die Karten für Kamera, Objekt, Wand, Lampe, Tür/Fenster, Zeichnung und jetzt auch Bild (#190) holen ihre Daten mit demselben achtzeiligen Block: am Zähler des Kamerabilds hängen, am Anfang des Schlüssels die Art erkennen, die Beschreibung abfragen, die Art noch einmal prüfen. Nur das Wort für die Art ist jeweils ein anderes. Dabei steht diese Schreibweise der Schlüssel („image:“ usw.) schon in der App (`plan/key.rs`). Die Karten schreiben sie ein zweites Mal hin.

Was es kostet: Jede neue Karte ist eine achte Abschrift. Ändert sich, wann eine Karte neu lesen muss oder wie ein Schlüssel aussieht, ändert man sieben Dateien. Das macht auch den offenen Fund 2 des zehnten Audits („Karten hängen am Zähler des Kamerabilds“) teurer.

Vorschlag: Eine gemeinsame Abfrage „Beschreibung des gewählten Elements dieser Art“, entweder in der Verbindungsschicht oder als kleiner gemeinsamer QML-Baustein. Jede Karte nennt dann nur noch ihre Art.

Aufwand klein bis mittel. Risiko gering: Jede der sieben Karten hat eigene Oberflächentests (`crates/app/tests/qml/tst_*card.qml`).

Beleg: `crates/app/qml/ImageCard.qml:16`, `crates/app/qml/CameraCard.qml:25`, `crates/app/qml/ObjectCard.qml:23`, `crates/app/qml/WallCard.qml:16`, `crates/app/qml/LightCard.qml:20`, `crates/app/qml/OpeningCard.qml:17`, `crates/app/qml/DrawingCard.qml:18`.

### 4. Zwei Tabellen-Ausgaben verschlucken einen möglichen Fehler, und die Grenze des ZIP-Formats wird zweimal geprüft
Seit #195 kann das ZIP-Schreiben ablehnen, wenn eine Datei zu groß ist (4 GiB oder mehr als 65535 Dateien). Die Ausgaben als ODS- und XLSX-Tabelle machen aus einer Ablehnung stillschweigend eine leere Datei. Ein Kommentar begründet das damit, dass eine Tabelle nie so groß wird. Alle anderen Ausgaben geben einen Fehler weiter.

Die Grenze steht außerdem zweimal: einmal als grobe Vorabprüfung (Anzahl und Summe der Dateien), einmal genau beim Schreiben jeder Datei.

Was es kostet: Ändert sich der Weg einmal (etwa Bilder in einer Tabelle), entsteht eine leere Datei ohne Meldung. Wird die Grenze geändert (zum Beispiel auf ZIP64 für große Archive), muss man beide Prüfungen finden.

Vorschlag: ODS und XLSX geben den Fehler weiter wie die übrigen Ausgaben. Die Vorabprüfung nimmt dieselben Grenzwerte wie das Schreiben, aus einer Stelle.

Aufwand klein. Risiko gering: `crates/export/tests/export.rs` und `crates/services/tests/folder.rs` (Ablehnung zu großer Ordner) decken es ab.

Beleg: `crates/export/src/lib.rs:144`, `crates/export/src/lib.rs:156`, `crates/export/src/zip.rs:37`, `crates/export/src/zip.rs:106`.

### 5. `ARCHITECTURE.md` nennt das ZIP-Archiv nicht unter den Aufgaben der Ausgabe
Seit #195 packt der Baustein „Ausgabe“ (`export/`) auch einen ganzen Projektordner als ZIP. Die neue Zeile in „Where new things go“ nennt das. Die Beschreibung des Bausteins oben zählt aber nur Plan, Shotliste und Ansichten auf.

Was es kostet: Wer in der Übersicht nachsieht, wo Archive gebaut werden, findet es nicht.

Vorschlag: Die Zeile `export/` nennt „ein Ordner als ZIP-Archiv“.

Aufwand klein, nur Text. Kein Risiko.

Beleg: `ARCHITECTURE.md:28`, `ARCHITECTURE.md:95`, `crates/export/src/lib.rs:108`.

### 6. Ein Überbleibsel aus #191: `rotate` gibt nur an `turned` weiter
Seit #191 dreht `Point::turned` im Datenmodell einen Punkt. Die Planänderungen behalten trotzdem ihre eigene Funktion `rotate`, die nichts anderes tut, als diese aufzurufen. Das Gruppendrehen ruft weiter `rotate`.

Was es kostet: Wenig. Man sieht aber zwei Namen für dasselbe und muss nachlesen, ob sie sich unterscheiden.

Vorschlag: Beide Aufrufer rufen `turned` direkt, `rotate` entfällt.

Aufwand klein. Risiko gering: Die Dreh- und Gruppentests in `crates/services/tests/` decken es ab.

Beleg: `crates/services/src/plan_edit/moving.rs:505`, `crates/services/src/plan_edit/moving.rs:501`, `crates/services/src/plan_edit/group.rs:67`.

### 7. Bestandsliste
**In Ordnung:** die flachen Listen (Feldnamen `name` in `field.rs`, Schlüssel `key` und `name` in `plan/key.rs`). Sie sind lang, weil es viele Elemente und Felder gibt, nicht weil sie viel tun. Die Erhöhungen durch #190 sind je ein Zweig und begründet. Die alten Einträge der Ausgabe (`jpeg.rs`, `layout.rs`, `scan.rs`) sind unverändert.

**Gewachsen wie erwartet:** `apply` (141 Zeilen) wächst mit jeder neuen Planänderung um einen Zweig. #190 hat die drei Bildänderungen geschickt in einem Zweig gebündelt.

**An der Grenze von 800 Zeilen:**
- `tst_planarea.qml` mit 797 Zeilen (vorher 782),
- `export/tests/export.rs` mit 799,
- `services/tests/library.rs` und `manage.rs` mit je 800,
- `services/tests/session.rs` mit 788,
- `model/src/plan.rs` mit 787,
- `app/src/bridge/view.rs` mit 779 (vorher 788).

Die nicht geprüfte `bridge.rs` wächst weiter (864 Zeilen) und hält sich an `ARCHITECTURE.md`: dort stehen nur Erklärungen, die Logik liegt in `bridge/<Bereich>.rs`.

Vorschlag: Nichts umbauen. Beim nächsten Test für die Planfläche `tst_planarea.qml` nach Werkzeugen teilen, so wie es `tst_planareadrop.qml` schon tut.

Beleg: `structure.baseline`, Commit `a8e2cb9`.

## Seit dem letzten Audit
Seit dem zwölften Audit wurden zwölf PRs gemergt: #186 bis #197. Aus dem zwölften Audit hat der Eigentümer die Funde 1 bis 4 gewählt; alle vier sind behoben. Fund 5 hat er nicht gewählt.

Zwölftes Audit:
- **Fund 1 (Bild als gedrehtes Rechteck, Rechnungen abgeschrieben): behoben (#191).** Möbel und Bild beschreiben sich als `TurnedRect`. Lage der Griffe, Ziehen und Einrasten laufen für beide über dieselben Funktionen. Rest siehe neuer Fund 6. Beleg: `crates/model/src/geometry.rs:63`, `crates/model/src/project.rs:284`, `crates/model/src/project.rs:499`, `crates/services/src/plan_edit/moving.rs:254`, `crates/services/src/plan_edit/moving.rs:272`.
- **Fund 2 (Griff → Element an drei Stellen): behoben (#189).** Die Klickerkennung folgt `Source::owner` und nennt nur Tür und Lampe als Ausnahmen. Beide Stellen der App fragen `Source::selected_itself`. Beleg: `crates/services/src/plan_edit/picking.rs:64`, `crates/stellprobe/src/display.rs:165`, `crates/app/src/selection.rs:85`, `crates/app/src/bridge/view.rs:369`.
- **Fund 3 (Anleitung für neue Elemente unvollständig): behoben (#187).** Beleg: `ARCHITECTURE.md:84`.
- **Fund 4 (Bildgröße auf zwei Wegen): behoben (#192).** Beleg: `crates/stellprobe/src/display.rs:278`, `crates/services/src/session/pick.rs:30`.
- **Fund 5 (Kopieren über einen gemeinsamen Ablauf): vom Eigentümer nicht gewählt.** Bleibt stehen und wird nicht neu gemeldet.

Zehntes Audit:
- **Fund 1 (Schnitt und Kamerablick auf eigenem Weg, `freeLook` schaltet und fragt): offen.** Beleg: `crates/app/src/bridge.rs:144`, `crates/app/src/bridge/free.rs:89`, `crates/app/qml/AppHeader.qml:367`, `crates/app/qml/FreeView.qml:140`.
- **Fund 2 (Karten hängen am Zähler des Kamerabilds): offen,** mit der Bildkarte jetzt sieben Karten (siehe neuer Fund 3). Beleg: `crates/app/src/bridge.rs:28`, `crates/app/src/bridge/view.rs:448`.
- **Fund 3 (Farbe eines Bildpunkts in App und Ausgabe je einmal): offen.** Beleg: `crates/app/src/plan/picture.rs:59`, `crates/export/src/style.rs:216`.
- **Fund 4 (Punkte von freier Form und gezeichneter Fläche zweimal gebaut): offen.** Beleg: `crates/services/src/plan_edit/drawings.rs:42`, `crates/services/src/plan_edit/moving/forms.rs:25`, `crates/services/src/plan_edit/moving.rs:222`, `crates/stellprobe/src/display/handles.rs:64`, `crates/stellprobe/src/display/handles.rs:67`.
- **Fund 5 („was eine Kamera sieht“ dreimal, „gewählte Kamera“ viermal): offen.** Beleg: `crates/app/src/bridge/export.rs:84`, `crates/app/src/bridge/shots.rs:57`, `crates/app/src/bridge/free.rs:92`, `crates/app/src/bridge/picture.rs:41`, `crates/services/src/session/views.rs:146`.

Achtes Audit:
- **Fund 1 (Seite und Scharnier der Tür in eigenen Listen der Karte): offen.** Beleg: `crates/app/qml/OpeningCard.qml:32`, `crates/app/src/object.rs:131`.
- **Fund 2 (Regeln der eigenen Bibliothek in Ablage und Sitzung): offen.** Beleg: `crates/services/src/project_edit/library.rs:78`, `crates/store/src/library.rs:106`.
- **Fund 3 (Regeln des Höhenwerkzeugs in der App): teilweise.** Beleg: `crates/app/src/tools.rs:276`, `crates/app/src/bridge/free.rs:282`.
- **Fund 4 (Vorlagen unter dem Namen der Bibliothek): offen.** Beleg: `crates/app/src/bridge/library.rs:19`.
- **Fund 5 (Punkt in Fläche zweimal): offen.** Beleg: `crates/model/src/library.rs:146`, `crates/stellprobe/src/hit.rs:110`.
- **Fund 6 (3D-Ansicht kopiert Bildteil des Kamerabilds): offen.** Beleg: `crates/app/qml/FreeView.qml:48`, `crates/stellprobe/src/eye.rs:78`.

Siebtes Audit:
- **Fund 1 (App errät Knick beim Ziehen): offen.** Beleg: `crates/app/src/bridge/draw.rs:22`.
- **Fund 2 (Feld „eigene Höhe oder Vorgabe“ dreimal): offen.** Beleg: `crates/app/qml/WallCard.qml:48`, `crates/app/qml/OpeningCard.qml:66`, `crates/app/qml/ObjectCard.qml:85`.
- **Fund 3 (Regeln für Türen und Fenster in Ablage und Sitzung): offen.** Beleg: `crates/store/src/lib.rs:470`, `crates/services/src/plan_edit/heights.rs:74`.
- **Fund 4 (Kamerabild färbt mit Kopie der Farbregeln von Kante): teilweise, unverändert.** Beleg: `crates/app/qml/ViewPalette.qml:13`.
- **Fund 5 (Raumhöhe eines neuen Drehorts in Modell und Fenster): offen.** Beleg: `crates/model/src/project.rs:192`, `crates/app/qml/LocationEditor.qml:14`.
- **Fund 6 (Dateinamen und `Hint` als Datenquelle der Karten): offen.** Beleg: `crates/app/src/bridge/hints.rs:44`.

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen.** Beleg: `crates/tui/src/fountain.rs:130`, `crates/script/src/parse.rs:9`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe): offen.** Beleg: `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen.** Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:337`, `crates/services/src/project_edit/templates.rs:171`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach): offen.** Beleg: `crates/services/src/plan_edit/group.rs:184`, `crates/services/src/project_edit/templates.rs:284`.

Viertes Audit:
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): Rest offen.** Beleg: `crates/cli/src/main.rs:186`, `crates/cli/src/main.rs:197`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** Beleg: `crates/app/qml/Workspace.qml`.
- **Fund 4 (angezeigte Werte in mehreren Listen ohne gegenseitige Prüfung): offen.** Beleg: `crates/app/src/bridge/view.rs:211`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert. Beleg: `crates/export/src/layout.rs:276`, `crates/export/src/scan.rs:85`.

Geprüft: Die Tests `crates/services/tests/folder.rs` (10) und `example.rs` (6) sind in diesem Audit grün gelaufen (`CARGO_TARGET_DIR=/home/arian/.cache/claude/klappe-audit13/target`). Die Strukturprüfung meldet 425 Dateien, 0 Verstöße und 31 Einträge in der Bestandsliste.

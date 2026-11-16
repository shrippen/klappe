# Code-Audit Klappe, 16.11.2026, Stand 42336c1

## Urteil
Das Repo ist wartbar und besser als beim elften Audit. Die vier gewählten Funde des elften Audits sind behoben (#179 bis #182). Dazu kommen drei ältere Funde:
- Ein Griff wird jetzt einmal beschrieben (neuntes Audit, #178).
- Das Einsetzen einer Vorlage ruft nicht mehr im Kreis (fünftes Audit, #111).
- Den Winkel beim Messen rechnet die Sitzung, nicht mehr die App (viertes Audit, #119).

Die Bestandsliste ist von 33 auf 31 Einträge geschrumpft. Das liegt allerdings nur daran, dass die Verbindungsdatei `bridge.rs` auf Entscheidung des Eigentümers nicht mehr geprüft wird. Neue Schwäche: Die freien Bilder auf dem Plan (#185) sind als gedrehtes Rechteck gebaut, wie ein Möbel. Die Rechnungen dafür sind aber neu geschrieben, statt die der Möbel zu benutzen (Fund 1). Außerdem hat #185 sechs Einträge der Bestandsliste angehoben, jeden mit Begründung.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 405 | 412 (ohne die Kopie von Kante, gezählt wie die Strukturprüfung) |
| Einträge in der Bestandsliste | 33 | 31. Weg sind die zwei Einträge von `bridge.rs`: Die Datei ist seit #183 ganz von der Prüfung ausgenommen. #185 hat sechs Einträge angehoben, jeweils um eins bis zwei Punkte Verzweigungen: die Schlüssel `key` und `name` (17 auf 19), `apply`, `copy`, `remove` und `target`. #184 hat `error_text` angehoben (18 auf 19). Jede Erhöhung ist im Commit begründet. |
| größte Datei (Zeilen) | 1011 (die Ablagetests `crates/store/tests/store.rs`) | 1011 (dieselbe). Ohne Tests ist die größte geprüfte Datei weiter `crates/export/src/jpeg.rs` mit 883 Zeilen. Die nicht mehr geprüfte `bridge.rs` ist von 817 auf 833 Zeilen gewachsen. |
| längste Funktion (Zeilen) | 138 (`apply` in `plan_edit.rs`) | 139 (dieselbe, eine Zeile für das neue Bild). Die Liste der Aufrufe `switched` wird nicht mehr gemessen, weil sie in `bridge.rs` steht. |

## Funde

### 1. Ein Bild auf dem Plan ist ein gedrehtes Rechteck wie ein Möbel, aber jede Rechnung dafür ist neu geschrieben
Seit #185 (LH-18) kann man Fotos und Skizzen frei auf den Plan legen, verschieben, drehen und an einem Griff breiter ziehen. Geometrisch ist ein Bild dasselbe wie ein Möbel: ein Rechteck um eine Mitte, um diese Mitte gedreht. Die Möbel haben dafür fertige Rechnungen. Das Bild benutzt keine davon, sondern hat von jeder eine eigene Abschrift:
- „Wo liegt ein Punkt des Rechtecks auf dem Plan“: im Datenmodell zweimal wortgleich.
- „Wo sitzt der Griff“: in Stellprobe zweimal, beim Bild nur für die Breite.
- „Wie ändert das Ziehen am Griff die Breite“: in der Sitzung zweimal. Die Bildfassung wiederholt die Möbelfassung Zeile für Zeile, nur ohne Tiefe.
- „Welcher Punkt rastet beim Ziehen ein“: zweimal.

Was es kostet: Ändert sich die Regel für das Ziehen, gilt sie danach nur für Möbel, und Bilder verhalten sich anders. Beispiele wären eine andere Mindestgröße oder Einrasten an der Kante statt am Griff. Jedes weitere Element dieser Art (etwa ein Schild oder eine Markierungsfläche) brächte eine vierte Abschrift.

Vorschlag: Die Rechnungen für „gedrehtes Rechteck“ werden so gefasst, dass sie Mitte, Breite und Drehung bekommen statt eines Möbels, und Möbel und Bild rufen dieselben auf.

Aufwand klein. Risiko gering: `crates/services/tests/object_grips.rs` (auch gedreht) und `crates/services/tests/images.rs` (auch gedreht) prüfen beide Seiten und sind in diesem Audit grün gelaufen.

Beleg: `crates/model/src/project.rs:265`, `crates/model/src/project.rs:471`, `crates/stellprobe/src/display/handles.rs:140`, `crates/stellprobe/src/display/handles.rs:268`, `crates/services/src/plan_edit/moving.rs:246`, `crates/services/src/plan_edit/moving.rs:279`, `crates/services/src/plan_edit/moving.rs:336`, `crates/services/src/plan_edit/moving.rs:341`.

### 2. Zu welchem Element ein Griff gehört und welcher Griff selbst gewählt wird, steht noch an drei Stellen
#178 hat die Zuordnung „Griff → sein Element“ an eine Stelle gebracht (`Source::owner`). Ebene und Auswahl lesen sie dort. Zwei Listen sind geblieben:
- **Die Klickerkennung der Sitzung** hat eine eigene Zuordnung „Griff → Element“ (`owner_of`). Sie lässt Türgriffe und Lampengriffe bewusst weg und fängt alles Übrige stumm mit „keins“ ab. `ARCHITECTURE.md` warnt selbst, dass der Compiler hier nicht meldet, wenn etwas fehlt. #185 musste den neuen Bildgriff in beide Zuordnungen eintragen.
- **Die App** führt zweimal die Liste der Griffe, die beim Klick selbst gewählt werden statt ihres Elements (Punkt einer Zeichnung, Punkt einer freien Form, Knick eines Wegs): einmal beim Klicken, einmal beim Aufräumen der Auswahl nach jeder Änderung.

Was es kostet: Bei einem neuen Griff kann man eine Stelle vergessen, ohne dass etwas meldet. Fehlt er in der Klickerkennung, fängt der Griff Klicks mitten im Element ab. Fehlt er in der zweiten Liste der App, verschwindet ein selbst gewählter Griff nach der nächsten Änderung aus der Auswahl.

Vorschlag: Die Klickerkennung nimmt die eine Zuordnung und nennt nur ihre zwei Ausnahmen ausdrücklich, und die App fragt an beiden Stellen dieselbe Funktion „wird dieser Griff selbst gewählt“.

Aufwand klein. Risiko gering: `crates/services/tests/grip_catch.rs`, `grip_layers.rs` und die Auswahltests der App decken es ab (Sitzungstests grün).

Beleg: `crates/stellprobe/src/display.rs:133`, `crates/services/src/plan_edit/picking.rs:63`, `crates/app/src/selection.rs:82`, `crates/app/src/bridge/view.rs:369`, `ARCHITECTURE.md:85`.

### 3. Die Anleitung für eine neue Art Element auf dem Plan nennt zwei Stellen nicht, die #185 ändern musste
`ARCHITECTURE.md` zählt auf, was man für eine neue Art Element anfasst, und sagt: Braucht eine Änderung mehr Stellen, ist das ein Fund. #185 musste zwei Stellen ändern, die in der Zeile „New kind of element on the plan“ fehlen:
- In der App braucht jedes Element einen eigenen Schlüssel mit Namen (`plan/key.rs`). Die Zeile für neue Griffe nennt die Datei, die für neue Elemente nicht.
- Die Ablage prüft beim Laden die Werte jedes Elements (`check_images` neben `check_notes` usw.). Die Zeile nennt nur Datenmodell und `FORMAT.md`.

Was es kostet: Wer der Anleitung folgt, vergisst die Ladeprüfung. Eine Projektdatei mit unsinnigen Werten lädt dann ohne Meldung.

Vorschlag: Die Zeile nennt beide Stellen.

Aufwand klein, nur Text. Kein Risiko.

Beleg: `ARCHITECTURE.md:84`, `crates/app/src/plan/key.rs:83`, `crates/store/src/lib.rs:383`, `crates/store/src/lib.rs:412`.

### 4. Stellprobe bekommt die Bildgröße auf zwei Wegen
Stellprobe liest keine Dateien. Wie groß ein Bild in Pixeln ist, muss ihr deshalb der Aufrufer sagen. Seit #185 bekommt sie dafür zwei Angaben:
- eine fertige Größe für das Hintergrundbild,
- eine Nachfrage für jedes andere Bild.

Die Sitzung beantwortet beide mit derselben Funktion. Die Nachfrage könnte die Größe des Hintergrunds also genauso liefern.

Was es kostet: Jeder Aufruf muss beides liefern. Die Ausgabe der Lampenliste reicht deshalb zweimal „nichts“ durch. Ein Aufrufer, der nur eines richtig füllt, zeigt Hintergrund und Bilder verschieden.

Vorschlag: Stellprobe fragt auch die Größe des Hintergrunds über die eine Nachfrage.

Aufwand klein. Risiko gering: Die Tests in `crates/stellprobe/tests/images.rs` und die Hintergrundtests der Sitzung decken beide ab.

Beleg: `crates/stellprobe/src/display.rs:257`, `crates/services/src/session/pick.rs:30`, `crates/services/src/session/pick.rs:61`, `crates/services/src/session/export.rs:224`.

### 5. Bestandsliste: Jede neue Art Element hebt dieselben sechs Einträge an
Die flachen Listen (Feldnamen `name` in `field.rs`, Schlüssel `key` und `name` in `plan/key.rs`) sind in Ordnung, wie sie sind. Sie sind lang, weil es viele Elemente gibt, nicht weil sie viel tun.

**Gewachsen:** Mit dem Bild (#185) sind sechs Einträge gewachsen, also mit jeder neuen Art Element:
- `apply`, die Verteilung aller Planänderungen,
- `copy`, das Kopieren,
- `remove`, das Löschen,
- `target`, die Zielsuche beim Ziehen,
- `key` und `name`.

Bei `copy` ist der Grund eine Wiederholung: Objekt, Lampe, Notiz und Bild werden mit demselben Ablauf kopiert (suchen, neue Nummer, verschieben, anhängen), viermal ausgeschrieben. Das Bild hat seinen Ablauf zudem in eine eigene Datei gelegt.

**In Ordnung:** `bridge.rs` ist seit #183 ausgenommen. `ARCHITECTURE.md` beschreibt das (nur Erklärungen, Logik in `bridge/<Bereich>.rs`), und die Datei hält sich daran.

**An der Grenze:** `bridge/view.rs` mit 788 Zeilen, das Datenmodell `model/src/plan.rs` mit 787, die Testdateien `services/tests/manage.rs` und `library.rs` mit je 800, `export/tests/export.rs` mit 799 und `tst_planarea.qml` mit 782.

Vorschlag: Das Kopieren der vier Elemente mit Nummer und Lage läuft über einen gemeinsamen Ablauf. Damit wächst `copy` mit dem nächsten Element nicht mehr.

Aufwand klein bis mittel. Risiko gering: Die Kopiertests in `crates/services/tests/` decken jedes Element ab.

Beleg: `structure.baseline`, `crates/services/src/plan_edit/group.rs:232`, `crates/services/src/plan_edit/images.rs:45`, Commit `31cb18a`.

## Seit dem letzten Audit
Seit dem elften Audit wurden neun PRs gemergt: #111, #119 und #178 bis #185. Aus dem elften Audit hat der Eigentümer alle fünf Funde gewählt:
- Funde 1 bis 4 sind behoben.
- Fund 5 ist durch seine Entscheidung erledigt.

Aus dem zehnten Audit wurde keiner gewählt oder abgelehnt.

Elftes Audit:
- **Fund 1 (vierte Liste beim Schließen): behoben (#179).** Das Schließen trägt einen leeren Satz Werte über dieselbe Funktion `show` ein. Damit wird auch der Name des Rasters geleert. Beleg: `crates/app/src/bridge/project.rs:73`, `crates/app/src/bridge/view.rs:213`.
- **Fund 2 (eigene Formatliste der Kommandozeile): behoben (#180).** Die Kommandozeile nimmt Namen und Endungen aus der Liste der Sitzung, und die CSV-Shotliste geht jetzt auch dort. Beleg: `crates/cli/src/main.rs:241`, `crates/cli/src/main.rs:253`.
- **Fund 3 (Blattgröße dreimal): behoben (#182),** mit neuem Test für die Kamerablicke. Beleg: `crates/export/src/layout.rs:77`.
- **Fund 4 (Regel „taugt als Vorbild“ zweimal): behoben (#181).** Beleg: `crates/stellprobe/src/setup.rs:68`, `crates/services/src/shot_edit.rs:232`.
- **Fund 5 (Bestandsliste, `bridge.rs`): erledigt durch Entscheidung des Eigentümers (#183).** Die Datei ist von der Prüfung ausgenommen, `ARCHITECTURE.md` begründet es. Siehe neuer Fund 5.

Zehntes Audit:
- **Fund 1 (Schnitt und Kamerablick auf eigenem Weg, `freeLook` schaltet und fragt): offen.** Beleg: `crates/app/src/bridge.rs:141`, `crates/app/src/bridge/free.rs:89`, `crates/app/qml/AppHeader.qml:367`, `crates/app/qml/AppHeader.qml:398`, `crates/app/qml/AppHeader.qml:408`, `crates/app/qml/FreeView.qml:140`.
- **Fund 2 (Karten hängen am Zähler des Kamerabilds): offen.** Das Schließen zählt ihn jetzt über `show` hoch statt über eine eigene Zeile. Beleg: `crates/app/src/bridge.rs:28`, `crates/app/src/bridge/view.rs:457`.
- **Fund 3 (Farbe eines Bildpunkts in App und Ausgabe je einmal): offen.** Beleg: `crates/app/src/plan/picture.rs:58`, `crates/export/src/style.rs:216`.
- **Fund 4 (Punkte von freier Form und gezeichneter Fläche zweimal gebaut): offen.** Beleg: `crates/services/src/plan_edit/drawings.rs:42`, `crates/services/src/plan_edit/moving/forms.rs:25`, `crates/services/src/plan_edit/moving.rs:221`, `crates/stellprobe/src/display/handles.rs:64`, `crates/stellprobe/src/display/handles.rs:67`.
- **Fund 5 („was eine Kamera sieht“ dreimal, „gewählte Kamera“ viermal): offen.** Beleg: `crates/app/src/bridge/export.rs:84`, `crates/app/src/bridge/shots.rs:57`, `crates/app/src/bridge/free.rs:92`, `crates/app/src/bridge/picture.rs:41`, `crates/services/src/session/views.rs:146`.

Neuntes Audit:
- **Fund 1 (neue Griffart an rund 14 Stellen): behoben (#178),** bis auf die Reste in neuem Fund 2. Eine Rolle `Handle` gilt für alle gleich aussehenden Griffe. `Source::owner` nennt das Element jedes Griffs. Ebene, Auswahl, Löschen und Drehen lesen es. `ARCHITECTURE.md` hat eine Zeile „New grip“. Beleg: `crates/stellprobe/src/display.rs:133`, `crates/app/src/selection.rs:47`.

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
- **Fund 3 (Regeln für Türen und Fenster in Ablage und Sitzung): offen.** Beleg: `crates/store/src/lib.rs:461`, `crates/services/src/plan_edit/heights.rs:74`.
- **Fund 4 (Kamerabild färbt mit Kopie der Farbregeln von Kante): teilweise, unverändert.** Beleg: `crates/app/qml/ViewPalette.qml:13`.
- **Fund 5 (Raumhöhe eines neuen Drehorts in Modell und Fenster): offen.** Beleg: `crates/model/src/project.rs:192`, `crates/app/qml/LocationEditor.qml:14`.
- **Fund 6 (Dateinamen und `Hint` als Datenquelle der Karten): offen.** Beleg: `crates/app/src/bridge/hints.rs:44`.

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen.** Beleg: `crates/tui/src/fountain.rs:130`, `crates/script/src/parse.rs:9`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe): offen.** Beleg: `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen.** Die dritte Stelle ist mit #111 umgezogen. Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:337`, `crates/services/src/project_edit/templates.rs:171`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach): offen.** Die Mitte der Gruppe steht weiter zweimal, mit #111 nur umgezogen. Beleg: `crates/services/src/plan_edit/group.rs:184`, `crates/services/src/project_edit/templates.rs:284`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): behoben (#111).** Das Einsetzen liegt jetzt ganz bei den Projektänderungen. Die Planänderungen rufen sie nicht mehr. Beleg: `crates/services/src/project_edit/templates.rs:13`.

Viertes Audit:
- **Fund 2 (Verbindungsdatei): erledigt durch Entscheidung des Eigentümers (#183),** siehe elftes Audit, Fund 5.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 5 (Winkel beim Messen in der App berechnet): behoben (#119).** Die Sitzung rechnet ihn, die App zeigt ihn nur. Beleg: `crates/services/src/measure.rs:8`, `crates/app/src/tools.rs:465`.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): Rest offen.** Die Kommandozeile schreibt für „keine solche Szene“ und „Szene ohne Plan“ weiter eigene englische Sätze. Beleg: `crates/cli/src/main.rs:186`, `crates/cli/src/main.rs:197`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** Beleg: `crates/app/qml/Workspace.qml`.
- **Fund 4 (angezeigte Werte in mehreren Listen ohne gegenseitige Prüfung): offen.** Die vierte Liste ist mit #179 weg, die drei übrigen (`collect`, `fill`, `show`) bleiben. Beleg: `crates/app/src/bridge/view.rs:213`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert. Beleg: `crates/export/src/layout.rs:276`, `crates/export/src/scan.rs:85`.

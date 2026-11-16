# Code-Audit Klappe, 16.11.2026, Stand 07f45c3

## Urteil
Das Repo bleibt wartbar. Seit dem neunten Audit sind zehn PRs gemergt worden (#140, #147, #156 bis #162, #164). Der größte war Schritt 6 der 3D-Ansicht (#140): Schnitt durch die Wände, Blick durch eine Kamera und die Ausgabe dieser Bilder. Er hat zwei neue Stellen gebracht, an denen dieselbe Sache doppelt läuft:
- Ob Schnitt und Kamerablick an sind, erfährt die Oberfläche auf einem Sonderweg statt auf dem üblichen (Fund 1). Im Review brauchte dieser Weg fünf Nachbesserungen.
- Wie ein Bildpunkt seine Farbe bekommt, steht in App und Ausgabe je einmal (Fund 3).

Besser geworden ist die Übersetzung der Fehlermeldungen: Sie hat jetzt 18 statt 26 Zweige. Die neuen Ausgaben stehen sauber in Ausgabe und Sitzung. Die Verbindungsdatei der App `bridge.rs` hat die Grenze von 800 Zeilen überschritten. Die Punkte der freien Formen (#160) bauen nach, was die Flächen der Zeichnungen schon können (Fund 4).

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 384 | 390 (ohne die Kopie von Kante, gezählt wie die Strukturprüfung) |
| Einträge in der Bestandsliste | 34 | 35. Neu ist die Verbindungsdatei `bridge.rs` mit 801 Zeilen. Gewachsen ist die Liste der Aufrufe `switched` (655 auf 660 Zeilen). Geschrumpft ist die Fehlerübersetzung `error_text` (26 auf 18 Zweige). |
| größte Datei (Zeilen) | 1483 (`crates/stellprobe/src/display.rs`) | 1483 (dieselbe, unverändert) |
| längste Funktion (Zeilen) | 138 (`apply` in `plan_edit.rs`) | 138 (dieselbe). `switched` mit 660 Zeilen ist weiter keine Funktion, sondern die Liste der Aufrufe. |

## Funde

### 1. Ob Schnitt und Kamerablick an sind, erfährt die Oberfläche auf einem eigenen Weg, und dieselbe Funktion schaltet und fragt
Für jeden Wert, den das Fenster zeigt, sieht `ARCHITECTURE.md` einen Weg vor: Die Sitzung gibt den Wert heraus, die Verbindung hält ihn als Eigenschaft, und die Oberfläche wird benachrichtigt, wenn er sich ändert. So läuft es zum Beispiel für „3D an oder aus“. Der Schnitt durch die Wände und der Blick durch eine Kamera gehen einen anderen Weg. Es gibt eine Funktion `freeLook`:
- Mit einem Befehl („cut“, „camera“) schaltet sie um.
- Mit einem leeren Befehl fragt sie nur.
- In beiden Fällen antwortet sie mit einer Wortliste wie „cut camera“.

Die Kopfzeile und die 3D-Ansicht zerlegen diese Liste an drei Stellen selbst. Sie fragen bei jedem neuen Bild von sich aus neu.

Was es kostet:
- **Ein Fehler, der schon passiert ist.** Fragen und Schalten in einer Funktion hat in #140 eine Endlosschleife ausgelöst: Das Fragen meldete eine Änderung, die Ansicht fragte deshalb wieder, bis das Programm abstürzte (Commit `a0b922c`). Vier weitere Commits waren nötig, damit ein veralteter Kamerablick richtig endet.
- **Teure Erweiterung.** Wer einen dritten Schalter dieser Art hinzufügt (etwa „Decke zeigen“), muss diesen Sonderweg verstehen und die Wortliste an drei Stellen in der Oberfläche auswerten.

Vorschlag: „Schnitt an“ und „Blick durch Kamera“ werden zwei gewöhnliche angezeigte Werte, wie „3D an“. Die Funktion `freeLook` schaltet nur noch.

Aufwand klein. Risiko gering: Der Szenariotest `free_look.qml`, die Kopfzeilentests in `tst_appheader.qml` und `services/tests/free_view.rs` (in diesem Audit grün gelaufen) prüfen das Verhalten.

Beleg: `ARCHITECTURE.md:84`, `crates/app/src/bridge.rs:139`, `crates/app/src/bridge/free.rs:89`, `crates/app/src/bridge/free.rs:100`, `crates/app/src/bridge/free.rs:117`, `crates/app/qml/AppHeader.qml:353`, `crates/app/qml/AppHeader.qml:384`, `crates/app/qml/AppHeader.qml:391`, `crates/app/qml/FreeView.qml:78`, `crates/app/qml/FreeView.qml:140`, `crates/app/src/bridge/view.rs:202` (so geht es für „3D an“), Commits `a0b922c`, `30b9b77`, `81f0cd3`, `6219148`, `a7f71fe`.

### 2. Acht Karten hängen an dem Zähler, der eigentlich nur das Kamerabild neu zeichnen soll
Eine Karte ist das kleine Feld mit den Werten des gewählten Elements (Wand, Tür, Möbel, Lampe, Kamera, Mark, Zeichnung). Sie muss nach jeder Änderung ihre Werte neu lesen. Bisher hing sie dafür am Plan. Seit #157 hängen alle acht Karten an `pictureRevision`. Dieser Zähler heißt „Stand des Bildes“, und seine Beschreibung sagt, er sei dafür da, das Kamerabild neu zu machen. Tatsächlich zählt er jede Änderung der Sitzung.

Was es kostet: Wer den Zähler so umbaut, dass er nur noch steigt, wenn sich das Bild ändert, lässt alle Karten veraltete Werte zeigen. Genau dieser Umbau wurde in #140 für den Zähler der 3D-Ansicht gemacht. Kein Test würde es als Ursache benennen; der Name führt in die Irre.

Vorschlag: Der Zähler bekommt einen Namen, der sagt, was er ist („Stand der Sitzung“), und eine Beschreibung, die beide Nutzer nennt.

Aufwand klein. Risiko gering: Die Kartentests `tst_*card.qml` prüfen, dass jede Karte nach einer Änderung neu liest.

Beleg: `crates/app/src/bridge.rs:27`, `crates/app/src/bridge/view.rs:454`, `crates/app/qml/CameraView.qml:13`, `crates/app/qml/WallCard.qml:19`, `crates/app/qml/OpeningCard.qml:20`, `crates/app/qml/ObjectCard.qml:26`, `crates/app/qml/LightCard.qml:21`, `crates/app/qml/CameraCard.qml:26`, `crates/app/qml/DrawingCard.qml:21`, `crates/app/qml/MarkCard.qml:40`, `crates/app/src/bridge/free.rs:287` (der Zähler der 3D-Ansicht, der nur noch bei echter Änderung steigt).

### 3. Wie ein Bildpunkt einer Ansicht seine Farbe bekommt, steht in App und Ausgabe je einmal
Ein Bild der 3D-Ansicht oder einer Kamera besteht aus Bildpunkten. Zu jedem Punkt weiß das Programm, was er zeigt (Wand, Boden, Figur), wie hell er beleuchtet ist und ob er auf einer Kante liegt. Seit #140 wird daraus an zwei Stellen eine Farbe gemacht, mit denselben Regeln:
- Leere Punkte bekommen die Grundfarbe.
- Kanten bekommen die volle Farbe.
- Flächen bekommen die Farbe mal die Helligkeit.
- Boden und Wände sind auf 20 % blasser gemischt.

Die eine Stelle ist der Bildschirm in der App, die andere die PNG-Ausgabe und die PDF-Seiten der Kamerablicke. Auch die Zahl 0,2 steht zweimal als eigene Konstante.

`ARCHITECTURE.md` sagt: Wie etwas aussieht, wird einmal entschieden; Bildschirm und Druck übersetzen nur in ihre Farben. Hier entscheiden beide selbst. Wer Wände etwas kräftiger haben will oder Kanten heller, muss beide Stellen gleich ändern. Sonst sieht der Ausdruck anders aus als der Bildschirm, und kein Test vergleicht die beiden.

Vorschlag: Stellprobe sagt je Bildpunkt, welche Farbrolle er hat und wie stark sie gemischt und abgedunkelt wird. App und Ausgabe setzen dafür nur noch ihre Farben ein.

Aufwand klein. Risiko gering: Die Tests in `app/src/plan/picture.rs` und `export/src/picture_png.rs` (in diesem Audit grün gelaufen) prüfen die Farben je Bildpunkt.

Beleg: `ARCHITECTURE.md:66`, `crates/app/src/plan/picture.rs:16`, `crates/app/src/plan/picture.rs:59`, `crates/app/src/plan/picture.rs:66`, `crates/app/src/plan/picture.rs:93`, `crates/export/src/style.rs:21`, `crates/export/src/style.rs:227`, `crates/export/src/style.rs:246`, `crates/export/src/picture_png.rs:32`, `crates/export/src/camera_pages.rs:23`.

### 4. Die Punkte einer freien Form und die Punkte einer gezeichneten Fläche sind zwei getrennte Bauten derselben Sache
Eine gezeichnete Fläche und eine freie Form sind beide ein geschlossener Umriss aus Punkten. Bei beiden kann man seit #154 und #160 dasselbe tun:
- einen Punkt ziehen,
- auf der Mitte einer Seite einen neuen Punkt einfügen,
- einen Punkt löschen.

Der Code dafür ist aber zweimal gebaut, mit verschiedenen Namen und verschiedenen Rechenwegen:
- **Namen:** Bei der Zeichnung heißen die Teile „Punkt“ und „Kante“ (`DrawingPoint`, `DrawingEdge`), bei der Form „Punkt“ und „Seite“ (`ObjectGrip::Point`, `ObjectGrip::Side`).
- **Rechenwege:** Ziehen, Einfügen und Löschen stehen einmal für Zeichnungen und einmal für Formen. Beide müssen nach jeder Änderung den umgebenden Kasten neu ausrechnen, damit der Rest an seinem Platz bleibt. Dafür benutzt die Zeichnung eine eigene Rechnung, die Form die des Modells. Dieselbe Kastenrechnung steht im Repo insgesamt fünfmal.
- **Mindestzahl:** Dass ein Umriss mindestens drei Punkte braucht, steht dreimal als eigene Zahl: im Modell, in der Sitzung und in der App.
- **Sonderfälle:** Die App behandelt beide Arten von Punkt je einzeln als Ausnahme, beim Klicken und beim Behalten der Auswahl. Die Sitzung tut dasselbe beim Ziehen.

Was es kostet: Jede neue Fähigkeit für Umrisspunkte muss zweimal gebaut werden, etwa Einrasten am Raster oder Punkte mit der Tastatur verschieben. Weicht eine Seite ab, verhalten sich Zeichnung und Möbel beim gleichen Griff verschieden.

Vorschlag: Die Sitzung bekommt eine Bearbeitung „Punkt eines geschlossenen Umrisses ziehen, einfügen, löschen, Drehung und Lage bleiben“, die beide benutzen. Das Modell hält die Mindestzahl und die Kastenrechnung einmal. Beide Arten heißen gleich.

Aufwand mittel. Risiko gering: `services/tests/drawing_points.rs`, `drawing_grips.rs` und `form_points.rs` (alle in diesem Audit grün gelaufen) sowie die Auswahltests in `app/src/selection.rs` prüfen das Verhalten.

Beleg: `crates/stellprobe/src/display.rs:176`, `crates/stellprobe/src/display.rs:186`, `crates/stellprobe/src/display.rs:192`, `crates/stellprobe/src/display/handles.rs:49`, `crates/stellprobe/src/display/handles.rs:52`, `crates/services/src/plan_edit/drawings.rs:42`, `crates/services/src/plan_edit/drawings.rs:84`, `crates/services/src/plan_edit/drawings.rs:119`, `crates/services/src/plan_edit/drawings.rs:151`, `crates/services/src/plan_edit/moving/forms.rs:25`, `crates/services/src/plan_edit/moving/forms.rs:60`, `crates/services/src/plan_edit/moving/forms.rs:73`, `crates/services/src/plan_edit/moving/forms.rs:90`, `crates/model/src/library.rs:46`, `crates/model/src/library.rs:150`, `crates/model/src/plan.rs:131`, `crates/services/src/plan_edit/group.rs:182`, `crates/services/src/project_edit/templates.rs:141`, `crates/services/src/plan_edit/add.rs:27`, `crates/app/src/tools.rs:48`, `crates/app/src/selection.rs:94`, `crates/app/src/bridge/view.rs:368`, `crates/services/src/plan_edit/moving.rs:604`.

### 5. „Was eine Kamera sieht“ wird in der Sitzung dreimal zusammengestellt, „die auf dem Plan gewählte Kamera“ in der App viermal
Für ein Bild aus Kamerasicht braucht die Sitzung drei Dinge: den Drehort der Szene, dessen Körper samt den eigenen Formen der Bibliothek und einen Blickpunkt. Das holt sie an drei Stellen je von Hand zusammen:
- für das Kamerabild auf dem Bildschirm und seine PNG-Ausgabe,
- für die Kamerablicke im PDF,
- für die 3D-Ansicht.

Zwei davon benutzen die vorhandene Funktion für den Drehort der Szene (`scene_location`) nicht.

In der App steht außerdem viermal derselbe Block „wenn genau eine Kamera gewählt ist, nimm ihre Kennung“: in der 3D-Ansicht, in der Ausgabe, in der Shotliste und im Kamerabild.

Was es kostet: Ändert sich, welche Formen ein Bild einbezieht, oder kommt eine weitere Quelle für die Kamera dazu (etwa die Kamera eines gewählten Marks), muss man drei beziehungsweise vier Stellen finden.

Vorschlag: Die Sitzung bekommt eine Funktion „Körper des Drehorts dieser Szene“, die alle drei Bilder benutzen. Die Auswahl in der App bekommt eine Abfrage „gewählte Kamera“.

Aufwand klein. Risiko gering: `services/tests/camera_picture.rs`, `free_view.rs`, `export/tests/shots.rs` und der Szenariotest `export_to.qml` decken es ab.

Beleg: `crates/services/src/session/views.rs:36`, `crates/services/src/session/views.rs:114`, `crates/services/src/session/export.rs:68`, `crates/services/src/session/free_view.rs:166`, `crates/services/src/session/free_view.rs:228`, `crates/app/src/bridge/free.rs:91`, `crates/app/src/bridge/export.rs:99`, `crates/app/src/bridge/shots.rs:44`, `crates/app/src/bridge/picture.rs:40`.

### 6. Bestandsliste: Die Verbindungsdatei der App ist über die Grenze gewachsen
Die Einschätzung des neunten Audits gilt weiter. Die Listen der Feldnamen (`name` in `field.rs`), der Aufrufe (`switched`) und der Schlüssel (`key`, `name` in `plan/key.rs`) sind flache Listen und in Ordnung, wie sie sind.

**Neu über der Grenze:** Die Verbindungsdatei `bridge.rs` hat 801 Zeilen. Der Eintrag wurde in #140 mit Grund angelegt: Die Aufteilung ist Fund 2 des vierten Audits und noch nicht gewählt. Jede neue Funktion des Fensters wächst hier um eine Erklärung. Die nächste wird die Datei weiter über die Grenze schieben, solange sie nicht geteilt ist.

**Unverändert an der Grenze:**
- Die Planansicht `display.rs` hat 1483 Zeilen, davon rund 900 Tests in derselben Datei.
- Die Datei der Planänderungen `plan_edit.rs` hat genau 800 Zeilen.
- Knapp darunter liegen `bridge/view.rs` mit 786 (statt 782), `services/tests/manage.rs` mit 800 und `library.rs` mit 799.

Die Teilung der Planansicht und der Planänderungen ist Fund 5 des neunten Audits und gewählt.

Vorschlag: Die Verbindungsdatei wird nach Bereichen geteilt (Plan, Karten, 3D, Ausgabe), wie es die Dateien unter `bridge/` schon sind. Fund 1 dieses Audits nimmt ihr eine Funktion ab.

Aufwand mittel. Risiko gering: Es wird nur verschoben, die Kopfzeilen- und Kartentests decken die Aufrufe ab.

Beleg: `structure.baseline`, `crates/app/src/bridge.rs:105`, `crates/app/src/bridge.rs:139`, `crates/stellprobe/src/display.rs:586`, `crates/services/src/plan_edit.rs:432`, `crates/app/src/bridge/view.rs`, Commit `133d2e6`.

## Seit dem letzten Audit
Seitdem wurden zehn PRs gemergt (#140, #147, #156 bis #162, #164). Aus dem neunten Audit hat der Eigentümer heute alle fünf Funde zum Beheben gewählt. Fund 2 ist als PR #163 in Arbeit und nicht gemergt; behoben ist noch keiner. Aus dem achten und den älteren Audits wurde keiner gewählt oder abgelehnt, und keiner ist behoben.

Neuntes Audit (alle gewählt):
- **Fund 1 (neue Griffart an rund 14 Stellen): offen, gewählt, etwas gewachsen.** #160 hat den Möbeln Griffe für Punkte und Seiten gegeben. Eine neue Rolle kam nicht dazu. Dafür kamen weitere Sonderfälle in Auswahl, Ziehen und Schlüsseln, siehe auch Fund 4. Beleg: `crates/stellprobe/src/look.rs:84`, `crates/services/src/session/layers.rs:123`, `crates/app/src/selection.rs:94`, `crates/app/src/bridge/view.rs:368`, `crates/services/src/plan_edit/moving.rs:604`, `crates/app/src/plan/key.rs:151`.
- **Fund 2 (verschiedene Untergrenzen beim Ziehen und Tippen): in Arbeit (#163, nicht gemergt).** Die neue Tempoeingabe (#147) hat ihre Grenzen 0,1 bis 15 m/s wieder in der Sitzung statt im Modell. Beleg: `crates/services/src/plan_edit/openings.rs:13`, `crates/services/src/plan_edit/moving.rs:23`, `crates/services/src/plan_edit/marks.rs:83`.
- **Fund 3 (Punkt mit dem Möbel mitdrehen fünfmal): offen, gewählt.** Beleg: `crates/stellprobe/src/display/handles.rs:128`, `crates/services/src/plan_edit/moving.rs:472`, `crates/services/src/plan_edit/moving.rs:488`, `crates/stellprobe/src/display/objects.rs:16`, `crates/stellprobe/src/scene.rs:450`.
- **Fund 4 (welche Ausgabe auf Papier druckt, zweimal in der App): offen, gewählt, gewachsen.** #140 hat zwei Ausgaben dazugebracht (3D-Ansicht und Kamerablick als PNG):
  - Ihre Dateiendung steht einmal im Dialog und einmal in der Verbindung.
  - Die Kommandozeile bietet sie nicht an, obwohl `ARCHITECTURE.md` für jedes neue Format eine Option dort verlangt.

  Die Behebung sollte beides einschließen. Beleg: `crates/app/qml/ExportDialog.qml:34`, `crates/app/qml/ExportDialog.qml:63`, `crates/app/src/bridge/export.rs:137`, `crates/app/src/bridge/export.rs:154`, `crates/cli/src/main.rs:76`, `ARCHITECTURE.md:85`.
- **Fund 5 (Bestandsliste): offen, gewählt.** Siehe neuer Fund 6.

Achtes Audit:
- **Fund 1 (Seite und Scharnier der Tür in eigenen Listen der Karte): offen.** Beleg: `crates/app/qml/OpeningCard.qml:31`, `crates/app/src/object.rs:131`.
- **Fund 2 (Regeln der eigenen Bibliothek in Ablage und Sitzung): offen.** Beleg: `crates/services/src/project_edit/library.rs:78`, `crates/store/src/library.rs:106`.
- **Fund 3 (Regeln des Höhenwerkzeugs in der App, Höhe als erfundener Planpunkt): offen.** Beleg: `crates/app/src/tools.rs:27`, `crates/app/src/bridge/free.rs:282`.
- **Fund 4 (Vorlagen unter dem Namen der Bibliothek): offen.** Beleg: `crates/app/src/bridge/library.rs:19`.
- **Fund 5 (Punkt in Fläche zweimal): offen.** Beleg: `crates/model/src/library.rs:141`, `crates/stellprobe/src/hit.rs:110`.
- **Fund 6 (3D-Ansicht kopiert Bildteil des Kamerabilds, Zoomschritt abgeschrieben): offen.** Beleg: `crates/app/qml/FreeView.qml:48`, `crates/stellprobe/src/eye.rs:78`.
- **Fund 7 (Bestandsliste): siehe neuer Fund 6.**

Siebtes Audit:
- **Fund 1 (App errät Knick beim Ziehen): offen.** Beleg: `crates/app/src/bridge/draw.rs:22`, `crates/services/src/plan_edit/moving.rs:168`.
- **Fund 2 (Feld „eigene Höhe oder Vorgabe“ dreimal): offen.** Beleg: `crates/app/qml/WallCard.qml:48`, `crates/app/qml/OpeningCard.qml:66`, `crates/app/qml/ObjectCard.qml:85`.
- **Fund 3 (Regeln für Türen und Fenster in Ablage und Sitzung): offen.** Beleg: `crates/store/src/lib.rs:437`, `crates/services/src/plan_edit/heights.rs:74`.
- **Fund 4 (Kamerabild färbt mit Kopie der Farbregeln von Kante): teilweise, unverändert.** Die Farbregel selbst steht seit #140 zusätzlich in der Ausgabe, siehe neuer Fund 3. Beleg: `crates/app/qml/ViewPalette.qml:13`, `crates/app/src/plan/picture.rs:16`.
- **Fund 5 (Raumhöhe eines neuen Drehorts in Modell und Fenster): offen.** Beleg: `crates/model/src/project.rs:188`, `crates/app/qml/LocationEditor.qml:14`.
- **Fund 6 (Dateinamen und `Hint` als Datenquelle der Karten): offen.** Beleg: `crates/app/src/bridge/hints.rs:43`, `crates/services/src/views.rs:390`.

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen.** Beleg: `crates/tui/src/fountain.rs:130`, `crates/script/src/parse.rs:9`.
- **Fund 2 (Breite von Türen und Fenstern in Ablage und Sitzung): offen**, Teil von Fund 2 des neunten Audits (#163). Beleg: `crates/store/src/lib.rs:442`, `crates/services/src/plan_edit/openings.rs:25`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe): offen.** Beleg: `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen.** Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:337`, `crates/services/src/plan_edit/template.rs:142`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach): offen.** Die Kastenmitte ist Teil des neuen Funds 4. Beleg: `crates/services/src/plan_edit/group.rs:182`, `crates/services/src/project_edit/templates.rs:141`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): offen.** Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/project_edit/templates.rs:11`.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): teilweise.** Der offene Rest ist Fund 4 des neunten Audits. Beleg: `crates/app/qml/ExportDialog.qml:34`, `crates/app/src/bridge/export.rs:137`.
- **Fund 2 (Verbindungsdatei, ein Aufruf je Mark-Feld): offen, jetzt über der Grenze** (801 Zeilen), siehe neuer Fund 6. Beleg: `crates/app/src/bridge.rs:105`.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 5 (Winkel beim Messen in der App berechnet): offen.** Beleg: `crates/app/src/tools.rs:472`, `crates/model/src/geometry.rs:44`.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): Rest offen.** Die Kommandozeile schreibt für „keine solche Szene“ und „Szene ohne Plan“ weiter eigene englische Sätze. Beleg: `crates/cli/src/main.rs:175`, `crates/cli/src/main.rs:186`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** Beleg: `crates/app/qml/Workspace.qml`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen.** `bridge/view.rs` ist von 782 auf 786 Zeilen gewachsen. Beleg: `crates/app/src/bridge/view.rs`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert. Beleg: `crates/export/src/layout.rs:264`, `crates/export/src/scan.rs:85`.

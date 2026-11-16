# Code-Audit Klappe, 16.11.2026, Stand 98783e4

## Urteil
Das Repo ist wartbar und etwas besser als beim zehnten Audit. Drei gewählte Funde des neunten Audits sind behoben:
- Ein Punkt dreht sich jetzt an einer einzigen Stelle mit dem Möbel mit.
- Welche Ausgaben es gibt, steht einmal in der Sitzung.
- Die Planansicht und die Planänderungen sind geteilt und liegen jetzt unter der Grenze von 800 Zeilen.

Die Bestandsliste ist von 35 auf 33 Einträge geschrumpft. Einen Eintrag hat die Teilung der Planansicht beseitigt, einen #174.

Schlechter geworden sind zwei Stellen:
- Die Verbindungsdatei der App `bridge.rs` ist weiter über der Grenze gewachsen. Wie im letzten Audit vorhergesagt, kommen mit jeder neuen Funktion des Fensters ein paar Zeilen dazu.
- Mit „Projekt schließen“ (#168) gibt es eine vierte Liste aller angezeigten Werte, die niemand gegen die anderen prüft (Fund 1).

Keiner der sechs Funde des zehnten Audits ist behoben. Zwei davon sind gewachsen.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 390 | 405 (ohne die Kopie von Kante, gezählt wie die Strukturprüfung) |
| Einträge in der Bestandsliste | 35 | 33. Weg sind zwei Einträge: die Planansicht `display.rs` als Datei (geteilt in #176) und `apply` in `edit.rs` (in #174 unter die Grenze gebracht). Die Zielsuche `target` steht unverändert, aber jetzt in `target.rs` statt in `moving.rs`; ihr Eintrag ist nur umgezogen. Gewachsen sind `bridge.rs` (801 auf 817 Zeilen), die Liste der Aufrufe `switched` (660 auf 676), die Feldnamen `name` in `field.rs` (42 auf 43 Zweige) und die Ablagetests `store.rs` (1010 auf 1011). Jede Erhöhung ist im Commit begründet. |
| größte Datei (Zeilen) | 1483 (`crates/stellprobe/src/display.rs`) | 1011 (die Ablagetests `crates/store/tests/store.rs`). Die größte Datei ohne Tests ist `crates/export/src/jpeg.rs` mit 883 Zeilen, unverändert. `display.rs` hat jetzt 587 Zeilen. |
| längste Funktion (Zeilen) | 138 (`apply` in `plan_edit.rs`) | 138 (dieselbe). `switched` mit 676 Zeilen ist weiter keine Funktion, sondern die Liste der Aufrufe. |

## Funde

### 1. Beim Schließen eines Projekts leert die App alle angezeigten Werte nach einer vierten, von Hand geführten Liste
Jeder Wert, den das Fenster aus dem Projekt zeigt, hat dafür schon drei Listen, die gleich gehalten werden müssen:
- `collect` holt die Werte,
- `fill` trägt sie beim Start ein,
- `show` trägt sie bei jeder Änderung ein.

Das ist Fund 4 des dritten Audits. Seit #168 („Projekt schließen“) gibt es eine vierte Liste, `show_closed`. Sie setzt rund dreißig Werte einzeln auf leer.

Sie stimmt schon heute nicht mit den anderen überein: Den Namen des Rasters (`grid_name`) setzt `show`, beim Schließen bleibt er stehen. Weder die Beschreibung in `ARCHITECTURE.md` („New value shown in the window“) noch der Hinweis über der Liste `Shown` nennt die vierte Stelle.

Was es kostet: Wer einen neuen Wert ins Fenster bringt, ändert nach der Anleitung drei Listen und vergisst die vierte. Nach dem Schließen zeigt eine Seite dann noch den Wert des alten Projekts. Kein Test vergleicht die Listen.

Vorschlag: Das Schließen trägt einen leeren Satz Werte über dieselbe Funktion `show` ein, statt die Werte selbst aufzuzählen.

Aufwand klein. Risiko gering: Der Szenariotest `close_project.qml` und `tst_projectguard.qml` prüfen das Schließen.

Beleg: `crates/app/src/bridge/project.rs:206`, `crates/app/src/bridge/view.rs:120`, `crates/app/src/bridge/view.rs:168`, `crates/app/src/bridge/view.rs:211`, `crates/app/src/bridge/view.rs:430` (`set_grid_name`, das beim Schließen fehlt), `ARCHITECTURE.md:84`.

### 2. Die Kommandozeile führt eine eigene Liste der Ausgabeformate und errät das Format aus der Dateiendung selbst
Seit #175 steht in der Sitzung einmal, welche Ausgaben es gibt, wie sie heißen und welche Dateiendung sie haben (`ExportFormat`). Die App liest diese Liste. Die Kommandozeile hat dagegen weiter drei eigene Stellen:
- eine eigene Liste der Formate,
- eine Übersetzung dieser Liste in die der Sitzung,
- eine eigene Tabelle „Endung → Format“.

Dass sie auseinanderlaufen, ist schon passiert: Die Shotliste als CSV gibt es im Fenster, in der Kommandozeile nicht. `--out liste.csv` endet mit „cannot tell the format“. `ARCHITECTURE.md` verlangt für jedes Format eine Option in der Kommandozeile, außer für eine Ansicht, die es dort nicht gibt.

Was es kostet: Ein neues Format muss in der Kommandozeile an drei Stellen nachgetragen werden. Fehlt eine, merkt es niemand.

Vorschlag: Die Kommandozeile nimmt die Namen und Endungen aus der Liste der Sitzung und lässt nur die 3D-Ansicht weg.

Aufwand klein. Risiko gering: `crates/cli/tests/export.rs` und die Tests in `export_format.rs` decken es ab (in diesem Audit grün gelaufen).

Beleg: `crates/cli/src/main.rs:81`, `crates/cli/src/main.rs:94`, `crates/cli/src/main.rs:264`, `crates/services/src/export_format.rs:50`, `crates/services/src/export_format.rs:64`, `ARCHITECTURE.md:85`.

### 3. Die Blattgröße in Hoch- oder Querformat rechnet die Ausgabe dreimal aus
Plan, Shotliste und Kamerablicke im PDF rechnen je selbst aus, wie groß das Blatt ist: Papiergröße holen, für Hoch oder Quer Breite und Höhe tauschen, in Punkte umrechnen. #170 (Shotliste hochkant) hat dafür eine eigene kleine Funktion gebaut. Die beiden älteren Stellen benutzen sie aber nicht.

Was es kostet: Ein neues Papierformat mit eigener Regel ändert man an drei Stellen. Ein Beispiel wäre ein Blatt, das immer quer liegt. Weicht eine Stelle ab, haben Plan und Shotliste im selben PDF verschiedene Blätter.

Vorschlag: Die Funktion aus der Shotliste wandert neben die Papiergrößen und wird von allen drei benutzt.

Aufwand klein. Risiko mittel:
- Für Plan und Shotliste prüfen Tests die Blattgröße: `crates/export/tests/export.rs:497` (`pdf_page_size_and_opacity`) und `crates/export/tests/shots.rs` (A4 quer, A4 hoch, Letter hoch). Beide sind grün.
- Für die Kamerablicke prüft kein Test die Blattgröße, nur Seitenzahl und Beschriftungen. Dort sollte vor dem Umbau ein Test für Hoch- und Querformat dazukommen.

Beleg: `crates/export/src/table.rs:151`, `crates/export/src/layout.rs:83`, `crates/export/src/camera_pages.rs:63`, `crates/export/src/layout.rs:68`, `crates/export/tests/shots.rs:556`.

### 4. Welcher Shot als Vorbild für ein Setup taugt, steht in Stellprobe und in der Sitzung je einmal
Seit #174 kann man einen Shot von Hand dem Setup eines anderen zuordnen. Ein solches Vorbild muss drei Bedingungen erfüllen:
- es muss den Shot geben,
- es ist nicht der Shot selbst,
- es ist nicht selbst von Hand zugeordnet.

Diese Regel steht zweimal mit derselben Bedingung:
- Stellprobe wendet sie beim Zählen der Setups an.
- Die Sitzung prüft damit die Eingabe.

Der Kommentar der Sitzung sagt selbst, dass sie nachbaut, was Stellprobe nimmt.

Was es kostet: Ändert sich die Regel, gilt sie nach einer halben Änderung nur halb. Ein Beispiel wäre, dass Ketten doch erlaubt werden. Dann nimmt die Sitzung eine Eingabe an, die Stellprobe still durch die errechnete Zuordnung ersetzt, oder umgekehrt.

Vorschlag: Stellprobe bietet die Prüfung „taugt dieser Shot als Vorbild“ an, und die Sitzung ruft sie auf.

Aufwand klein. Risiko gering: Die Tests in `crates/stellprobe/src/setup.rs` und `crates/services/tests/shots.rs` decken beide Seiten ab (grün).

Beleg: `crates/stellprobe/src/setup.rs:68`, `crates/services/src/shot_edit.rs:136`, `crates/services/src/shot_edit.rs:142`.

### 5. Bestandsliste: Die Verbindungsdatei der App wächst mit jeder Funktion des Fensters
Die Einschätzung des zehnten Audits gilt weiter. Die Listen der Feldnamen (`name` in `field.rs`), der Aufrufe (`switched`) und der Schlüssel (`key`, `name` in `plan/key.rs`) sind flache Listen. Sie sind in Ordnung, wie sie sind.

**Gewachsen:** Drei PRs haben `bridge.rs` je um eine Erklärung von fünf bis sechs Zeilen vergrößert, auf jetzt 817 Zeilen:
- #168: Projekt schließen,
- #173: Auswahl aufheben,
- #174: Setups zur Wahl.

Jeder hat den Grund genannt. Das ist genau, was das zehnte Audit vorhergesagt hat. Solange die Datei nicht geteilt ist, hebt jede neue Funktion des Fensters die Grenze weiter an.

**Behoben:** Die Planansicht (`display.rs`, jetzt 587 Zeilen) und die Planänderungen (`plan_edit.rs`, jetzt 745 Zeilen) sind geteilt (#176).

**An der Grenze:** `bridge/view.rs` mit 786 Zeilen, die Testdateien `crates/services/tests/manage.rs` und `library.rs` mit je 800 Zeilen und `crates/export/tests/export.rs` mit 799.

Vorschlag: Die Verbindungsdatei wird nach Bereichen geteilt (Plan, Karten, Shots, 3D, Ausgabe, Projekt), wie es die Dateien unter `bridge/` schon sind. Das ist Fund 2 des vierten Audits. Fund 1 des zehnten Audits nimmt ihr zusätzlich eine Funktion ab.

Aufwand mittel. Risiko gering: Es wird nur verschoben. Die Kopfzeilen-, Karten- und Szenariotests decken die Aufrufe ab.

Beleg: `structure.baseline`, `crates/app/src/bridge.rs:28`, `crates/app/src/bridge.rs:141`, `crates/app/src/bridge.rs:611`, Commits `03f4b31`, `2c8560b`, `4fd5105`.

## Seit dem letzten Audit
Seit dem zehnten Audit wurden zehn PRs gemergt (#167 bis #176). Aus dem neunten Audit hatte der Eigentümer alle fünf Funde gewählt. Vier davon sind jetzt behoben. Der fünfte (Fund 1) ist in Arbeit, aber noch nicht als PR gestellt. Aus dem zehnten Audit wurde keiner gewählt oder abgelehnt.

Zehntes Audit:
- **Fund 1 (Schnitt und Kamerablick auf eigenem Weg, `freeLook` schaltet und fragt): offen.** Beleg: `crates/app/src/bridge.rs:141`, `crates/app/src/bridge/free.rs:89`, `crates/app/qml/AppHeader.qml:367`, `crates/app/qml/AppHeader.qml:398`, `crates/app/qml/AppHeader.qml:408`, `crates/app/qml/FreeView.qml:140`.
- **Fund 2 (Karten hängen am Zähler des Kamerabilds): offen, gewachsen.** Auch das Schließen eines Projekts (#168) zählt jetzt diesen Zähler hoch. Beleg: `crates/app/src/bridge.rs:28`, `crates/app/src/bridge/view.rs:455`, `crates/app/src/bridge/project.rs:244`.
- **Fund 3 (Farbe eines Bildpunkts in App und Ausgabe je einmal): offen.** Beleg: `crates/app/src/plan/picture.rs:16`, `crates/export/src/style.rs:227`.
- **Fund 4 (Punkte von freier Form und gezeichneter Fläche zweimal gebaut): offen.** #171 behandelt beide beim Anzeigen der Seitengriffe gleich (eine gemeinsame Mindestlänge), der Rest ist unverändert. Beleg: `crates/services/src/plan_edit/drawings.rs:42`, `crates/services/src/plan_edit/moving/forms.rs:25`, `crates/services/src/plan_edit/moving.rs:220`, `crates/stellprobe/src/display/handles.rs:64`, `crates/stellprobe/src/display/handles.rs:67`.
- **Fund 5 („was eine Kamera sieht“ dreimal, „gewählte Kamera“ viermal): offen.** #175 hat die Ausgabe der App umgebaut, den Block „gewählte Kamera“ aber behalten. Beleg: `crates/app/src/bridge/export.rs:84`, `crates/app/src/bridge/shots.rs:57`, `crates/app/src/bridge/free.rs:92`, `crates/app/src/bridge/picture.rs:41`, `crates/services/src/session/views.rs:36`.
- **Fund 6 (Bestandsliste): siehe neuer Fund 5.**

Neuntes Audit (alle gewählt):
- **Fund 1 (neue Griffart an rund 14 Stellen): gewählt, in Arbeit (noch kein PR), gewachsen.** #171 („der nächste Griff gewinnt“) hat zwei weitere Stellen gebracht:
  - eine eigene Liste aller Griffrollen in der Sitzung,
  - eine eigene Zuordnung „Griff → sein Element“.

  Diese Zuordnung steht in der App schon einmal (`whole`), dort mit zwei Griffarten mehr. Beleg: `crates/services/src/plan_edit/picking.rs:38`, `crates/services/src/plan_edit/picking.rs:69`, `crates/app/src/selection.rs:47`, `crates/stellprobe/src/look.rs:84`, `crates/services/src/session/layers.rs:123`, `crates/app/src/bridge/view.rs:371`.
- **Fund 2 (Untergrenzen beim Ziehen und Tippen): behoben (#163).** Die Grenzen der Tempoeingabe stehen weiter in der Sitzung. Beleg: `crates/services/src/plan_edit/marks.rs:82`.
- **Fund 3 (Punkt mit dem Möbel mitdrehen fünfmal): behoben (#169).** Alle fünf Stellen benutzen jetzt `SetObject::placed` im Modell. Beleg: `crates/model/src/project.rs:437`.
- **Fund 4 (welche Ausgabe auf Papier druckt, Endungen, Kommandozeile): behoben (#175).** Die Liste steht in der Sitzung, Dialog und Verbindung lesen sie, und die Kommandozeile bietet den Kamerablick an. Die eigene Formatliste der Kommandozeile ist neuer Fund 2. Beleg: `crates/services/src/export_format.rs:4`, `crates/app/qml/ExportDialog.qml:22`.
- **Fund 5 (Bestandsliste): behoben (#176)** für Planansicht und Planänderungen. Siehe neuer Fund 5.

Achtes Audit:
- **Fund 1 (Seite und Scharnier der Tür in eigenen Listen der Karte): offen.** Beleg: `crates/app/qml/OpeningCard.qml:32`, `crates/app/src/object.rs:131`.
- **Fund 2 (Regeln der eigenen Bibliothek in Ablage und Sitzung): offen.** Beleg: `crates/services/src/project_edit/library.rs:78`, `crates/store/src/library.rs:106`.
- **Fund 3 (Regeln des Höhenwerkzeugs in der App): teilweise.** Die Untergrenze kommt seit #163 aus dem Modell, der Rest ist offen. Beleg: `crates/app/src/tools.rs:274`, `crates/app/src/bridge/free.rs:282`.
- **Fund 4 (Vorlagen unter dem Namen der Bibliothek): offen.** Beleg: `crates/app/src/bridge/library.rs:19`.
- **Fund 5 (Punkt in Fläche zweimal): offen.** Beleg: `crates/model/src/library.rs:146`, `crates/stellprobe/src/hit.rs:110`.
- **Fund 6 (3D-Ansicht kopiert Bildteil des Kamerabilds): offen.** Beleg: `crates/app/qml/FreeView.qml:48`, `crates/stellprobe/src/eye.rs:78`.

Siebtes Audit:
- **Fund 1 (App errät Knick beim Ziehen): offen.** Beleg: `crates/app/src/bridge/draw.rs:22`.
- **Fund 2 (Feld „eigene Höhe oder Vorgabe“ dreimal): offen.** Beleg: `crates/app/qml/WallCard.qml:48`, `crates/app/qml/OpeningCard.qml:66`, `crates/app/qml/ObjectCard.qml:85`.
- **Fund 3 (Regeln für Türen und Fenster in Ablage und Sitzung): offen.** Beleg: `crates/store/src/lib.rs:437`, `crates/services/src/plan_edit/heights.rs:74`.
- **Fund 4 (Kamerabild färbt mit Kopie der Farbregeln von Kante): teilweise, unverändert.** Siehe Fund 3 des zehnten Audits. Beleg: `crates/app/qml/ViewPalette.qml:13`.
- **Fund 5 (Raumhöhe eines neuen Drehorts in Modell und Fenster): offen.** Beleg: `crates/model/src/project.rs:188`, `crates/app/qml/LocationEditor.qml:14`.
- **Fund 6 (Dateinamen und `Hint` als Datenquelle der Karten): offen.** Beleg: `crates/app/src/bridge/hints.rs:43`.

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen.** Beleg: `crates/tui/src/fountain.rs:130`, `crates/script/src/parse.rs:9`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe): offen.** Beleg: `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen.** Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:337`, `crates/services/src/plan_edit/template.rs:142`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach): offen.** Beleg: `crates/services/src/plan_edit/group.rs:182`, `crates/services/src/project_edit/templates.rs:141`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): in Arbeit (#111).** Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/project_edit/templates.rs:11`.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): behoben** mit Fund 4 des neunten Audits (#175), bis auf die Kommandozeile (neuer Fund 2).
- **Fund 2 (Verbindungsdatei): offen, weiter gewachsen** (817 Zeilen), siehe neuer Fund 5.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 5 (Winkel beim Messen in der App berechnet): in Arbeit (#119).** Beleg: `crates/app/src/tools.rs:468`, `crates/model/src/geometry.rs:44`.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): Rest offen.** Die Kommandozeile schreibt für „keine solche Szene“ und „Szene ohne Plan“ weiter eigene englische Sätze. Beleg: `crates/cli/src/main.rs:210`, `crates/cli/src/main.rs:221`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** Beleg: `crates/app/qml/Workspace.qml`.
- **Fund 4 (angezeigte Werte in mehreren Listen ohne gegenseitige Prüfung): offen, gewachsen.** Es gibt jetzt eine vierte Liste, siehe neuer Fund 1. Beleg: `crates/app/src/bridge/view.rs:211`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert. Beleg: `crates/export/src/layout.rs:264`, `crates/export/src/scan.rs:85`.

# Code-Audit Klappe, 15.11.2026, Stand 2fd2d85

## Urteil
Das Repo ist wartbar und steht etwa so da wie beim sechsten Audit. Gut ist vor allem der neue Baustein für das Kamerabild. Das Paket, das Dreiecke in Pixel umsetzt (`raster/`), hängt von nichts ab, nur Stellprobe benutzt es, und `ARCHITECTURE.md` beschreibt es. Jedes Paket hängt weiter nur von den Paketen ab, die die Datei erlaubt.

Weitere Fortschritte: Die Grenzwerte für Lampen stehen jetzt einmal im Modell (#120). Die Ablage fasst ihre Prüfungen beim Laden wie empfohlen in einem eigenen Schritt zusammen. Die größte Datei und das Ziel einer Verschiebung sind kleiner geworden.

Auf der Gegenseite steht dreierlei:
- Von den 14 offenen Funden der letzten Audits ist nur dieser eine behoben, obwohl der Eigentümer alle beheben lassen wollte.
- Mit den Höhen im Set und den Wegen mit mehreren Knicken sind neue Doppelungen entstanden. Eine davon hat schon einen Fehler verursacht (Fund 1).
- Die Datei mit den Planänderungen steht 8 Zeilen vor der Grenze.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 316 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung) | 338 |
| Einträge in der Bestandsliste | 32 | 32. Gewachsen sind vier Einträge: `apply` in `plan_edit.rs`, `check`, die Liste der Feldnamen `name` in `field.rs` und `switched` in `bridge.rs`. Geschrumpft sind drei: `target` in `moving.rs`, `display.rs` und der Test der Ablage. Neu ist keiner. |
| größte Datei (Zeilen) | 1559 (`crates/stellprobe/src/display.rs`) | 1476 (dieselbe Datei, die Griffe stehen jetzt in `display/handles.rs`) |
| längste Funktion (Zeilen) | 134 (`apply` in `plan_edit.rs`) | 138 (dieselbe). `switched` ist von 558 auf 576 Zeilen gewachsen. Es ist weiter keine Funktion, sondern die Liste der Aufrufe der Verbindung zur Oberfläche. |

## Funde

### 1. Ob ein Ziehen am Weg einen neuen Knick gemacht hat, errät die App, statt es von der Sitzung zu erfahren
Ein Weg einer Figur oder Kamera hat Griffe. Zieht man an einem Griff zwischen zwei Knicken, entsteht ein neuer Knick, und der soll danach ausgewählt sein (LH-21). Ob wirklich einer entstanden ist, entscheidet die Sitzung, also der Teil, über den jede Oberfläche das Projekt ändert. Sie hat dafür zwei Regeln: Ein Griff, der nicht bewegt wurde, macht keinen Knick, und ein Knick, der nach dem Einrasten am Raster wieder auf dem Griff landet, auch nicht.

Die App kennt diese Regeln nicht. Sie sucht selbst im Projekt, zu welchem Weg der Griff gehört, und zählt dessen Knicke vor und nach der Änderung. Das Suchen gibt es in der Sitzung schon (`grip_target`). Welcher Knick danach ausgewählt wird, rechnet die Auswahl der App noch einmal getrennt für Figuren und Kameras aus.

Was das kostet, war schon beim Entstehen zu sehen. Die erste Fassung der App nahm an, dass immer ein Knick entsteht. Ließ die Sitzung nach dem Einrasten keinen entstehen, war ein alter Knick ausgewählt, und die Entf-Taste löschte ihn (Commit `71c0969`). Ändert sich eine Regel der Sitzung oder bekommt ein drittes Element Wege, muss die App wieder von Hand nachziehen. Das Terminalprogramm oder ein späterer Editor hätten dasselbe Problem.

Vorschlag: Die Änderung in der Sitzung meldet zurück, welchen Knick sie gemacht hat (oder keinen), und die App wählt genau diesen aus. Das Zählen in der App entfällt dann. Aufwand klein. Risiko gering: `services/tests/curves.rs`, die Tests in `app/src/selection.rs` und in `app/src/bridge/draw.rs` decken beide Seiten ab.

Beleg: `crates/app/src/bridge/draw.rs:21`, `crates/app/src/bridge/draw.rs:388`, `crates/app/src/selection.rs:139`, `crates/app/src/selection.rs:148`, `crates/services/src/plan_edit/moving.rs:129`, `crates/services/src/plan_edit/moving.rs:199`, `crates/services/src/plan_edit/moving.rs:395`, Commits `7eb9ad5`, `71c0969`, `d83fffe`.

### 2. Das Feld „eigene Höhe oder Vorgabe“ steht dreimal fast gleich in drei Karten
Wand, Tür oder Fenster und Möbelstück zeigen jetzt Höhen (LH-90). Bei vier Feldern gibt es eine Vorgabe: Höhe der Wand, Brüstung und Sturz einer Öffnung, Unterkante eines Möbelstücks. Alle vier verhalten sich gleich:
- Die Vorgabe steht grau im leeren Feld, ein eigener Wert steht als Text darin.
- Ein Feld, das man nicht angefasst hat, schickt beim Übernehmen nichts. So wird die Vorgabe nie als eigener Wert festgeschrieben.
- Ein geleertes Feld kehrt zur Vorgabe zurück.

Diese Regeln stehen in jeder Karte als eigene Funktionen, mit verschiedenen Namen und kleinen Abweichungen. Soll sich das Verhalten ändern, etwa mit einem Knopf „Vorgabe wiederherstellen“, muss man es an drei Stellen gleich ändern. Die nächste Höhe mit Vorgabe, etwa eine Figurengröße je Szene, würde die vierte Kopie. Den Baustein für ein beschriftetes Feld (`LabeledField.qml`) hat dieselbe Änderung schon angelegt, aber nur für die Beschriftung.

Vorschlag: Ein Feld „Wert mit Vorgabe“ übernimmt Platzhalter, eigenen Wert und „unberührt schickt nichts“ und ruft die Verbindung mit dem Feldnamen auf. Die drei Karten setzen es nur ein. Aufwand klein. Risiko gering: `tst_wallcard.qml`, `tst_openingcard.qml` und `tst_objectcard.qml` prüfen das Verhalten jeder Karte.

Beleg: `crates/app/qml/WallCard.qml:48`, `crates/app/qml/WallCard.qml:54`, `crates/app/qml/OpeningCard.qml:59`, `crates/app/qml/OpeningCard.qml:70`, `crates/app/qml/ObjectCard.qml:85`, `crates/app/qml/ObjectCard.qml:91`, `crates/app/qml/LabeledField.qml:8`.

### 3. Regeln für Türen und Fenster stehen in Ablage und Sitzung je für sich, und ein Begriff hat zwei Namen
Mit den Höhen ist zur offenen Breitenprüfung (Fund 2 des sechsten Audits) eine zweite Regel dazugekommen, die doppelt steht: Die Brüstung muss unter dem Sturz liegen. Die Ablage prüft das beim Öffnen eines Projekts. Die Sitzung prüft es beim Eintippen von Brüstung und Sturz noch einmal, jeweils mit eigenem Vergleich. Ändert sich die Regel, etwa „mindestens 10 cm Luft“, muss man zwei Stellen in zwei Paketen ändern. Sonst öffnet sich ein Projekt, das man so nicht eintippen könnte, wie es bei der Breite schon heute der Fall ist.

Dazu hat dieselbe Sache zwei Namen. Das Paar Brüstung und Sturz heißt im Modell `sill_head_m`. In Stellprobe gibt es eine Funktion `opening_span`, die nur diese eine Funktion aufruft. Ablage und Sitzung nehmen je einen der beiden Namen. Gleichzeitig heißt im Modell `span_m` etwas anderes, nämlich wo die Öffnung entlang der Wand beginnt und endet. Wer nach „span“ sucht, findet zwei verschiedene Dinge.

Vorschlag: Eine Prüfung „ist eine brauchbare Öffnung“ kommt ins Modell. Sie umfasst Breite, Brüstung, Sturz und deren Reihenfolge, und Ablage und Sitzung rufen sie auf. Die Weiterleitung `opening_span` entfällt. Das behebt Fund 2 des sechsten Audits gleich mit. Aufwand klein. Risiko gering: `store/tests/heights.rs`, `store/tests/openings.rs`, `services/tests/set_heights.rs` und `services/tests/openings.rs` decken beide Seiten ab.

Beleg: `crates/store/src/lib.rs:380`, `crates/store/src/lib.rs:428`, `crates/store/src/lib.rs:431`, `crates/services/src/plan_edit/heights.rs:75`, `crates/services/src/plan_edit/heights.rs:93`, `crates/services/src/plan_edit/check.rs:196`, `crates/stellprobe/src/height.rs:29`, `crates/model/src/project.rs:334`, `crates/model/src/project.rs:342`.

### 4. Das Kamerabild färbt sich mit einer Kopie der Farbregeln von Kante
Der Plan wird von einem Baustein aus Kante gezeichnet. Der kennt die Regeln, wie aus einem Rollennamen eine Farbe wird. Ein unbekannter Name wird zur Textfarbe, und eine Fläche „getönt“ hat ein Fünftel der Farbstärke. Das neue Kamerabild färbt seine Pixel nicht über Kante, sondern selbst. Die App baut im Fenster eine eigene Farbtabelle aus Kantes Werten, und die Verbindung mischt in Rust nach eigenen Regeln. Beide Regeln stehen dort noch einmal, die Tönung als eigene Konstante 0.2 mit dem Kommentar „wie im Plan“.

Ändert Kante die Tönung oder bekommt es einen neuen Farbnamen, sehen Plan und Kamerabild dasselbe Element verschieden aus, ohne dass ein Test es merkt. Laut `agent.md` gehört, was der Oberfläche fehlt, zuerst nach Kante.

Vorschlag: Kante bietet die Farbtabelle und die Tönung als eine Stelle an, etwa eine Funktion, die die Palette als Werte liefert, oder eine Konstante für die Tönung. Kamerabild und Plan lesen dort. Aufwand mittel, weil zuerst das Repo Kante geändert und dann übernommen wird. Risiko gering: Die Tests in `app/src/plan/picture.rs` und `tst_cameraview.qml` prüfen die Farben.

Beleg: `crates/app/src/plan/picture.rs:16`, `crates/app/src/plan/picture.rs:37`, `crates/app/src/plan/picture.rs:40`, `crates/app/qml/CameraView.qml:40`, `crates/app/qml/Kante/KanteShapeCanvas.qml:97`, `crates/app/qml/Kante/KanteShapeCanvas.qml:207`.

### 5. Die Raumhöhe eines neuen Drehorts steht in Modell und Fenster je für sich
Ein neuer Drehort bekommt die Raumhöhe 2,60 m. Das Modell hat dafür eine Konstante. Das Formular im Fenster schreibt denselben Wert noch einmal als feste Zahl in sein Feld. Soll die Vorgabe zum Beispiel 3 m werden, muss man beide Stellen ändern. Sonst öffnet das Fenster mit 2,60 m, und ein Drehort, der auf anderem Weg entsteht, bekommt 3 m. Breite und Tiefe (10 m und 8 m) stehen schon länger nur im Formular.

Vorschlag: Die Sitzung liefert die Vorgaben eines neuen Drehorts aus dem Modell, und das Formular zeigt sie an. Aufwand klein. Risiko gering: `tst_projectdialog.qml` prüft das Formular.

Beleg: `crates/model/src/project.rs:183`, `crates/app/qml/LocationEditor.qml:14`, `crates/app/qml/LocationEditor.qml:25`.

### 6. Drei Dateinamen sagen nicht, was in der Datei steht, und `ARCHITECTURE.md` verschweigt deshalb eine Stelle
In der Verbindung zur Oberfläche heißt die Datei für Szenen, Drehorte und Figuren `script.rs`, also „Drehbuch“. Mit dem Drehbuch hat sie nichts zu tun, Fountain liegt im Paket `script/`. Die Farbgebung des Kamerabilds liegt in `plan/picture.rs`, obwohl sie nicht den Plan betrifft. Neben ihr gibt es zwei weitere Dateien namens `picture`.

Die Daten jeder Karte (Wand, Tür, Möbelstück, Kamera, Lampe) kommen über die Funktion für Tooltips (`describe`, Datei `hints.rs`, „Data for the tooltips“) und den Typ `Hint`. Deshalb nennt `ARCHITECTURE.md` diese Stelle in der Zeile „neues Feld auf einer Karte“ nicht. Die Höhen-Änderung (#128) musste trotzdem `Hint` erweitern, in `services/src/views.rs` und `session/views.rs`, und eine Ladeprüfung in der Ablage ergänzen. Wer der Tabelle folgt, findet beide Stellen nicht.

Vorschlag: `bridge/script.rs` umbenennen, etwa nach Projektlisten, und die Farbgebung des Kamerabilds aus `plan/` herausnehmen. Die Zeile in `ARCHITECTURE.md` nennt außerdem `Hint` und die Ladeprüfung, oder `Hint` bekommt einen Namen, der Karten einschließt. Aufwand klein. Risiko gering: Es ändert sich nur der Name, und der Übersetzer prüft jede Stelle.

Beleg: `crates/app/src/bridge/script.rs:1`, `crates/app/src/plan.rs:13`, `crates/app/src/plan/picture.rs:1`, `crates/app/src/bridge/hints.rs:1`, `crates/app/qml/WallCard.qml:23`, `crates/services/src/views.rs:293`, `crates/services/src/session/views.rs:214`, `ARCHITECTURE.md:83`.

### 7. Bestandsliste: Die Datei der Planänderungen steht 8 Zeilen vor der Grenze
Die Einschätzung der letzten Audits gilt weiter: Die Verteiler, die nach der Art eines Elements verzweigen, trennen sich an einer natürlichen Naht. Die Liste der Feldnamen (`name` in `field.rs`, jetzt 42 Zweige) und die Liste der Aufrufe der Verbindung (`switched`) sind flache Listen und in Ordnung, wie sie sind. Neu ist Folgendes.

**Besser geworden:** Das Ziel einer Verschiebung (`target`) ist mit den Wegen mit mehreren Knicken von 130 auf 113 Zeilen geschrumpft. Die Griffe eines Weges haben eine eigene Funktion bekommen. Die Datei der Planansicht hat ihre Griffe in eine eigene Datei abgegeben (1476 statt 1559 Zeilen).

**Gewachsen:** `apply` ist von 134 auf 138 Zeilen und von 42 auf 43 Zweige gewachsen, `check` von 22 auf 23. Die Höhen werden dort aber richtig gelöst: `apply` gibt vier der fünf Höhen an eine eigene Datei `heights.rs` weiter. Nur die Höhe eines Möbelstücks läuft noch auf eigenem Weg durch `apply`, durch einen eigenen Aufruf der Verbindung (`setObjectHeight`) und durch eine eigene Funktion der App. Legt man sie mit den anderen Höhen zusammen, wird `apply` kürzer.

**Knapp unter der Grenze von 800 Zeilen:** Das sind jetzt zehn Dateien:
- Die Datei der Planänderungen `plan_edit.rs`: 792 statt 755 Zeilen. Die nächste Art von Element bringt sie über die Grenze.
- Die Verbindung `bridge.rs`: 786 statt 764.
- Der Plan der App `app/src/plan.rs`: 784 statt 762.
- Die Tests der Projektverwaltung und der Bibliothek: je 800.
- Die Tests der Ausgabe: 797.
- Die Tests der Sitzung: 788.
- Das Modell des Plans `model/src/plan.rs`: 787.
- `bridge/view.rs`: geschrumpft von 777 auf 764.

Neue Tests liegen inzwischen in eigenen Dateien nach Thema (`set_heights.rs`, `curves.rs`, `camera_picture.rs`), das ist gut.

Vorschlag: aus `plan_edit.rs` die Verwaltung des Bereichs (`Scope`) oder die Bestimmung der Ebene je Element in eine eigene Datei nehmen, bevor die nächste Art von Element kommt. Aufwand klein je Stelle. Risiko gering: `services/tests/plan_edit.rs`, `openings.rs`, `set_heights.rs`, `curves.rs` und `multisel.rs` decken die Verteiler ab.

Beleg: `crates/services/src/plan_edit.rs:439`, `crates/services/src/plan_edit.rs:520`, `crates/services/src/plan_edit.rs:571`, `crates/app/src/bridge.rs:333`, `crates/app/src/object.rs:92`, `crates/services/src/plan_edit/moving.rs:11`, `crates/services/src/field.rs:96`, `structure.baseline`.

## Seit dem letzten Audit
Der Eigentümer hat am 15.11.2026 entschieden, dass alle Funde behoben werden. Abgelehnt hat er keinen. Seitdem wurden zehn PRs gemergt (#120 bis #129). Davon behebt einer einen Fund (#120), und einer kürzt die Bestandsliste (#123).

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen, noch nicht begonnen.** Beleg: `crates/tui/src/fountain.rs:121`, `crates/tui/src/fountain.rs:151`, `crates/script/src/parse.rs:9`, `crates/script/src/parse.rs:15`.
- **Fund 2 (Breite von Türen und Fenstern in Ablage und Sitzung je für sich geprüft): offen, noch nicht begonnen.** Mit den Höhen ist eine zweite doppelte Regel dazugekommen, siehe neuer Fund 3. Beleg: `crates/store/src/lib.rs:380`, `crates/services/src/plan_edit/check.rs:196`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe je für sich): offen, noch nicht begonnen.** Beleg: `crates/services/src/session/shots.rs:11`, `crates/export/src/shots.rs:21`.
- **Fund 4 (Bestandsliste): teilweise.** `target` ist geschrumpft (#123), `apply` und `check` sind weiter gewachsen, siehe neuer Fund 7.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen, noch nicht begonnen.** Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:318`, `crates/services/src/plan_edit/template.rs:142`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach geschrieben): offen, noch nicht begonnen.** Beleg: `crates/services/src/plan_edit/group.rs:186`, `crates/services/src/project_edit/templates.rs:141`, `crates/services/src/plan_edit/template.rs:115`, `crates/services/src/plan_edit/group.rs:304`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): offen.** PR #111 ist weiter nicht gemergt. Beide Bereiche verweisen weiter gegenseitig aufeinander. Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/project_edit/templates.rs:11`.
- **Fund 6 (Bestandsliste): offen**, siehe neuer Fund 7.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): offen, noch nicht begonnen.** Beleg: `crates/app/qml/ExportDialog.qml:22`, `crates/app/src/bridge/export.rs:82`, `crates/cli/src/main.rs:64`.
- **Fund 2 (Verbindungsdatei knapp unter der Grenze, ein Aufruf je Mark-Feld): offen, noch nicht begonnen.** `bridge.rs` ist von 764 auf 786 Zeilen gewachsen. Beleg: `crates/app/src/bridge.rs:362`, `crates/app/src/bridge.rs:400`.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen, noch nicht begonnen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 4 (Grenzwerte einer Lampe an zwei Stellen geprüft): behoben (#120).** Ablage und Sitzung rufen dieselben Prüfungen aus dem Modell. Beleg: `crates/model/src/light_preset.rs:9`, `crates/store/src/light.rs:42`, `crates/services/src/light_edit.rs:52`.
- **Fund 5 (Winkel beim Messen in der App berechnet): offen, noch nicht begonnen.** Beleg: `crates/app/src/tools.rs:372`, `crates/model/src/geometry.rs:44`.
- **Fund 6 (Bestandsliste): offen.** `plan_edit.rs` ist von 755 auf 792 Zeilen gewachsen, siehe neuer Fund 7.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): behoben (#112), mit kleinem Rest.** Für „keine solche Szene“ und „Szene ohne Plan“ schreibt die Kommandozeile weiter eigene englische Sätze. Beleg: `crates/cli/src/main.rs:173`, `crates/cli/src/main.rs:184`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen, noch nicht begonnen.** Beleg: `crates/app/qml/Workspace.qml:114`, `crates/app/qml/Workspace.qml:125`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen, noch nicht begonnen.** Die Datei ist von 777 auf 764 Zeilen geschrumpft, weil die alte Kameraansicht entfallen ist. Beleg: `crates/app/src/bridge/view.rs`.
- **Fund 5 (Bestandsliste): offen, noch nicht begonnen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert.

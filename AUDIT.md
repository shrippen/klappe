# Code-Audit Klappe, 16.11.2026, Stand 1461f75

## Urteil
Das Repo ist wartbar und steht etwas besser da als beim siebten Audit. Die großen Neuerungen (freie 3D-Ansicht, Höhen ziehen, Raumwerkzeug, freie Form, Bauformen) liegen dort, wo `ARCHITECTURE.md` sie haben will. Die Rechnungen für 3D stehen in Stellprobe, die Sitzung reicht sie weiter, und die Oberfläche fragt nur. Die Verbindungsdatei war kurz über der Grenze (851 Zeilen in #131) und wurde im nächsten PR wieder darunter gebracht (#132). Die freie Form prüft ihren Umriss an einer Stelle im Modell, und das Weitergeben der Vorlagen benutzt die Prüfung der Ablage, statt sie zu kopieren.

Dagegen stehen drei Dinge:
- Von den offenen Funden der letzten Audits ist keiner behoben.
- Die neue Wahl, wie eine Tür aufgeht, verstößt gegen eine Regel aus `ARCHITECTURE.md` (Fund 1).
- Eine Regel der Bibliothek steht nur in der Sitzung, nicht beim Laden in der Ablage (Fund 2).

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 338 | 365 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung) |
| Einträge in der Bestandsliste | 32 | 32. Gewachsen sind zwei Einträge: die Liste der Aufrufe der Verbindung `switched` (576 auf 650 Zeilen, in vier PRs) und die Übersetzung der Fehler `error_text` (24 auf 25 Zweige). Geschrumpft ist die Datei der Planansicht `display.rs`. Ein Eintrag kam in #131 dazu und ging in #132 wieder weg. |
| größte Datei (Zeilen) | 1476 (`crates/stellprobe/src/display.rs`) | 1447 (dieselbe Datei; die Beschriftung der Möbel steht jetzt in `display/objects.rs`) |
| längste Funktion (Zeilen) | 138 (`apply` in `plan_edit.rs`) | 138 (dieselbe). `switched` ist auf 650 Zeilen gewachsen. Es ist weiter keine Funktion, sondern die Liste der Aufrufe. |

## Funde

### 1. Wie eine Tür aufgeht, wählt das Fenster aus eigenen Listen statt aus dem Modell
Eine Tür hat jetzt eine Seite, zu der sie aufgeht (links oder rechts), und ein Ende, an dem ihr Scharnier sitzt (Anfang oder Ende, LH-10). Für solche Auswahllisten schreibt `ARCHITECTURE.md` vor: Die Namen stehen einmal beim Typ im Modell, und die Oberfläche bekommt die Liste von dort. Für Posen, Lichtarten, Linien und Ebenen ist das so gebaut. Jeder Typ hat dort eine vollständige Liste und Namen, und die Verbindung reicht alle Listen in einem Paket an das Fenster (`choices_json`).

Die Tür macht es anders. Das Modell hat nur die beiden Typen, ohne Liste und ohne Namen. Die Karte im Fenster schreibt sich die Namen `left`, `right`, `start`, `end` selbst hin. Die App übersetzt sie in einer eigenen Tabelle zurück. Kommt eine dritte Möglichkeit dazu, etwa eine Schiebe- oder Pendeltür, muss man Modell, App-Tabelle und Karte von Hand gleich ändern. Vergisst man eine Stelle, bietet die Karte die neue Wahl nicht an, oder die Wahl kommt nicht an, und kein Test merkt es.

Vorschlag: Seite und Scharnier bekommen im Modell ihre Liste und Namen wie die Posen. Die Verbindung gibt sie mit den anderen Listen aus, und die Karte liest sie dort. Aufwand klein. Risiko gering: `tst_openingcard.qml`, der Szenariotest `door_swing.qml` und `services/tests/openings.rs` prüfen das Verhalten.

Beleg: `ARCHITECTURE.md:68`, `crates/model/src/project.rs:327`, `crates/model/src/project.rs:343`, `crates/app/qml/OpeningCard.qml:31`, `crates/app/qml/OpeningCard.qml:32`, `crates/app/qml/OpeningCard.qml:177`, `crates/app/src/object.rs:124`, `crates/app/src/object.rs:131`, `crates/app/src/bridge/view.rs:274`, `crates/app/src/bridge/view.rs:292`.

### 2. Was ein Eintrag der eigenen Bibliothek sein darf, prüfen Ablage und Sitzung je mit eigener Liste, und eine Regel fehlt beim Laden
Ein eigener Bibliothekseintrag ist ein Möbelstück, das der Nutzer im Projekt abgelegt hat. Seit #138 darf kein Eintrag die Art `form` haben, denn die gehört der freien Form. Sonst zählte jede freie Form als dieser Eintrag. Diese Regel steht nur in der Sitzung, also in dem Teil, über den jede Oberfläche das Projekt ändert. Die Ablage, die die Datei `bibliothek.toml` beim Öffnen liest, kennt sie nicht.

Dazu prüfen beide Teile die übrigen Regeln je für sich:
- Die Sitzung prüft Art, Größe und Höhe mit eigenen Zeilen.
- Die Ablage prüft dieselben drei und zusätzlich Umriss, Bauform und doppelte Art.

Ein Projekt, dessen Bibliothek von Hand oder von einer anderen Fassung einen Eintrag `form` bekommen hat, öffnet sich deshalb ohne Meldung. Nennt dieser Eintrag eine Bauform, bekommt jede freie Form im 3D-Bild dessen Körper. Das ist genau der Fehler, den #138 beheben sollte. Ändert sich eine Regel, muss man zwei Listen in zwei Paketen gleich ändern.

Bei den Vorlagen ist das schon richtig gelöst: Die Sitzung ruft dort die Prüfung der Ablage (`check_template`) auf.

Vorschlag: Die Regeln eines Eintrags kommen als eine Prüfung ins Modell, die reservierte Art eingeschlossen. Ablage und Sitzung rufen sie auf, wie bei den Grenzwerten der Lampen (#120). Aufwand klein. Risiko gering: `store/tests/library.rs`, `services/tests/library.rs` und `services/tests/free_form.rs` decken beide Seiten ab.

Beleg: `crates/model/src/library.rs:6`, `crates/store/src/library.rs:106`, `crates/store/src/lib.rs:121`, `crates/services/src/project_edit/library.rs:78`, `crates/services/src/project_edit/library.rs:107`, `crates/services/src/library.rs:33`, `crates/services/src/project_edit/templates.rs:207`, `crates/store/src/template.rs:118`, Commit `8d25cca`.

### 3. Die Regeln des Höhenwerkzeugs stehen in der App, und der 3D-Blick schleust die Höhe als erfundenen Planpunkt durch
Zieht man mit dem Höhenwerkzeug an einem Möbelstück oder einer Wand, wird es höher oder niedriger (LH-95). Zwei Regeln dafür stehen in der App statt in Sitzung oder Modell:
- Die neue Höhe wird auf ganze Zentimeter gerundet.
- Sie wird nie kleiner als 5 cm.

Eine getippte Höhe kennt beide Regeln nicht. 1,234 m bleibt 1,234 m, und 1 cm wird angenommen. Das Terminalprogramm oder ein späterer Editor müssten die Regeln noch einmal schreiben.

Der 3D-Blick benutzt dazu das Ziehen des Plans mit. Er übergibt die Höhe als Weg von einem erfundenen Punkt (0, Höhe) nach (0, 0), und das Ziehen liest die Höhe wieder aus dem Abstand der Punkte. Wer das Ziehen im Plan ändert, muss wissen, dass zwei seiner Zahlen in 3D gar keine Punkte sind.

Ähnlich rechnet das Tippen einer Wandlänge die Richtung zum Zeiger in der App selbst aus. Das Modell hat dafür schon Abstand und Zwischenpunkt (`distance_m`, `lerp`). Das ist dieselbe Art Abweichung wie Fund 5 des vierten Audits (Winkel beim Messen).

Vorschlag: Die Sitzung bekommt eine Änderung „um so viel anheben“, die Rundung und Mindesthöhe kennt. Plan und 3D übergeben ihr nur den Hub. Die Wandlänge nimmt die Hilfen des Modells. Aufwand klein bis mittel. Risiko gering: `app/src/tools/tests.rs` und die Szenariotests `height_drag.qml` und `typed_measure.qml` prüfen das Verhalten.

Beleg: `crates/app/src/tools.rs:23`, `crates/app/src/tools.rs:27`, `crates/app/src/tools.rs:278`, `crates/app/src/tools.rs:229`, `crates/app/src/bridge/draw.rs:160`, `crates/app/src/bridge/free.rs:230`, `crates/app/src/object.rs:92`, `crates/model/src/geometry.rs:16`, `crates/model/src/geometry.rs:21`.

### 4. Das Weitergeben von Vorlagen läuft unter dem Namen der Bibliothek
Vorlagen (eine Aufstellung von Figuren und Kameras) lassen sich jetzt als Datei weitergeben (#141). Dafür gibt es keinen eigenen Aufruf. Die Aufrufe „Bibliothek importieren“ und „Bibliothek exportieren“ bekommen einen Schalter, ein Wort `objects` oder `templates`. Jedes andere Wort, auch ein Tippfehler, gilt still als Möbel. Der Kommentar dazu nennt das eine Rücksicht auf früher, und vor dem ersten Release ist so etwas nicht nötig. Der Fehler „nichts weiterzugeben“ heißt bei beiden `EmptyLibrary`.

Wer etwas am Weitergeben der Vorlagen sucht, findet es unter „Library“. Ein drittes Weitergebbares, etwa Lampen, würde einen dritten Wert desselben Schalters bekommen. Der Grund im PR ist, die Liste der Aufrufe (`switched`) nicht wachsen zu lassen. Zwei eigene Aufrufe kosten dort aber nur rund zehn Zeilen.

Vorschlag: eigene Aufrufe für Vorlagen, ohne Schalter und ohne stille Rückfallregel, und ein Fehlername, der beide meint. Aufwand klein. Risiko gering: `tst_planmenu.qml`, der Szenariotest `templates.qml` und `services/tests/templates.rs` prüfen das Verhalten.

Beleg: `crates/app/src/bridge/library.rs:12`, `crates/app/src/bridge/library.rs:22`, `crates/app/src/bridge.rs:515`, `crates/app/src/bridge.rs:521`, `crates/app/qml/Main.qml:159`, `crates/app/qml/Main.qml:174`, `crates/services/src/session/templates.rs:77`, `crates/services/src/error.rs:62`.

### 5. Ob ein Punkt in einer Fläche liegt, steht zweimal
Für die freie Form prüft das Modell jetzt, ob ein Punkt in ihrem Umriss liegt. Daran entscheidet es, wo der Name steht. Stellprobe hat dieselbe Prüfung schon länger für das Anklicken von Flächen und das Fangen mit dem Lasso, mit einer eigenen Schleife. Auch der Schnittpunkt einer Kante mit einer Waagrechten ist an beiden Stellen dieselbe Formel. Wird eine der beiden Prüfungen für Grenzfälle verbessert, etwa einen Punkt genau auf der Kante, verhalten sich Beschriftung und Anklicken verschieden.

Vorschlag: Stellprobe nimmt die Prüfung des Modells, und die eigene entfällt. Aufwand klein. Risiko gering: Die Tests in `model/src/library.rs` und `stellprobe/src/hit.rs` decken beide Seiten ab.

Beleg: `crates/model/src/library.rs:126`, `crates/model/src/library.rs:141`, `crates/stellprobe/src/hit.rs:110`, `crates/stellprobe/src/hit.rs:16`, `crates/stellprobe/src/hit.rs:70`.

### 6. Die 3D-Ansicht kopiert den Bildteil des Kamerabilds und hält den Zoomschritt des Kerns von Hand nach
Die 3D-Ansicht und das Kamerabild zeigen beide ein Pixelbild mit Namen darüber und einem Hinweis, wenn es nichts zu zeigen gibt. Diesen Teil hat die 3D-Ansicht aus dem Kamerabild kopiert: Bildfläche, Namen mit Umrandung, Hinweistext. Soll ein Name anders aussehen, etwa größer oder mit Hintergrund, muss man beide Dateien gleich ändern. Die Farbtabelle teilen sich beide schon (`ViewPalette.qml`), das ist gut.

Dazu schreibt die 3D-Ansicht den Zoomschritt des Kerns (0,85 je Radstufe) als eigene Zahl 1,18 ab, damit eine Fingergeste gleich weit zoomt wie das Mausrad. Ändert sich der Schritt im Kern, zoomen Geste und Rad verschieden.

Vorschlag: Ein Baustein „Bild mit Namen“ wird von beiden Ansichten benutzt. Den Zoomschritt einer Geste rechnet der Kern, oder die Verbindung gibt ihn heraus. Aufwand klein. Risiko gering: `tst_cameraview.qml` und `tst_freeview.qml` prüfen beide Ansichten.

Beleg: `crates/app/qml/CameraView.qml:63`, `crates/app/qml/CameraView.qml:87`, `crates/app/qml/CameraView.qml:96`, `crates/app/qml/FreeView.qml:102`, `crates/app/qml/FreeView.qml:111`, `crates/app/qml/FreeView.qml:130`, `crates/app/qml/FreeView.qml:46`, `crates/stellprobe/src/eye.rs:72`.

### 7. Bestandsliste: Die Liste der Aufrufe wächst mit jedem PR, eine Datei steht genau an der Grenze
Die Einschätzung der letzten Audits gilt weiter. Die Verteiler, die nach der Art eines Elements verzweigen, trennen sich an einer natürlichen Naht. Die Liste der Feldnamen (`name` in `field.rs`) und die Liste der Aufrufe (`switched`) sind flache Listen.

**Besser geworden:**
- Die Planansicht hat die Beschriftung der Möbel in eine eigene Datei abgegeben (1447 statt 1476 Zeilen).
- Die Datei der Planänderungen ist von 792 auf 770 Zeilen geschrumpft.
- Die Verbindungsdatei war mit der 3D-Ansicht über der Grenze und wurde im nächsten PR durch Auslagern ihres Zustands wieder darunter gebracht (#132).

**Gewachsen:** `switched` ist in vier PRs von 576 auf 650 Zeilen gewachsen. Laut PR verlangt das Werkzeug für die Verbindung zu Qt, dass alle Aufrufe in einem Block stehen. Eine flache Liste ist in Ordnung, aber jede neue Funktion des Fensters verlängert sie. Fund 4 zeigt, dass schon versucht wird, an ihr zu sparen, und das führt zu schlechteren Namen.

**An oder knapp unter der Grenze von 800 Zeilen:**
- Der Plan der App `app/src/plan.rs`: genau 800 statt 784. Davon sind 465 Zeilen Tests in derselben Datei.
- Die Tests der Projektverwaltung (`services/tests/manage.rs`) mit 800 und der Bibliothek (`services/tests/library.rs`) mit 799.
- Die Verbindung `bridge.rs` mit 791 und `bridge/view.rs` mit 777 (statt 764).

Vorschlag: Die Tests von `app/src/plan.rs` kommen in eine eigene Datei, wie es bei `tools/tests.rs` schon ist. Neue Tests der Bibliothek kommen in eine Datei nach Thema. Aufwand klein. Risiko gering: Es werden nur Tests verschoben.

Beleg: `crates/app/src/bridge.rs:105`, `crates/app/src/plan.rs:334`, `crates/app/src/plan.rs:800`, `crates/services/tests/manage.rs`, `crates/services/tests/library.rs`, `crates/app/src/bridge/view.rs`, `structure.baseline`.

## Seit dem letzten Audit
Seitdem wurden elf PRs gemergt (#130 bis #141, ohne #140). Der Eigentümer hat aus dem siebten Audit noch keinen Fund zum Beheben gewählt. Abgelehnt hat er keinen. Behoben ist keiner.

Siebtes Audit:
- **Fund 1 (App errät, ob ein Ziehen am Weg einen Knick gemacht hat): offen.** Beleg: `crates/app/src/bridge/draw.rs:22`, `crates/app/src/bridge/draw.rs:515`, `crates/services/src/plan_edit/moving.rs:129`.
- **Fund 2 (Feld „eigene Höhe oder Vorgabe“ dreimal): offen.** Beleg: `crates/app/qml/WallCard.qml:48`, `crates/app/qml/OpeningCard.qml:66`, `crates/app/qml/OpeningCard.qml:84`, `crates/app/qml/ObjectCard.qml:85`.
- **Fund 3 (Regeln für Türen und Fenster in Ablage und Sitzung, zwei Namen für Brüstung und Sturz): offen.** Beleg: `crates/store/src/lib.rs:405`, `crates/store/src/lib.rs:459`, `crates/services/src/plan_edit/heights.rs:74`, `crates/stellprobe/src/height.rs:29`.
- **Fund 4 (Kamerabild färbt mit einer Kopie der Farbregeln von Kante): teilweise.** Die Farbtabelle steht jetzt einmal in `ViewPalette.qml`, für Kamerabild und 3D-Ansicht. Sie liegt aber weiter außerhalb von Kante, und die Tönung 0,2 steht weiter zweimal. Beleg: `crates/app/qml/ViewPalette.qml:13`, `crates/app/src/plan/picture.rs:16`, `crates/app/qml/Kante/KanteShapeCanvas.qml:97`.
- **Fund 5 (Raumhöhe eines neuen Drehorts in Modell und Fenster): offen.** Beleg: `crates/model/src/project.rs:183`, `crates/app/qml/LocationEditor.qml:14`.
- **Fund 6 (Dateinamen und `Hint` als Datenquelle der Karten): offen.** Die Tür hat `Hint` wieder erweitert. Beleg: `crates/app/src/bridge/script.rs:1`, `crates/app/src/plan/picture.rs:1`, `crates/services/src/views.rs:390`, `ARCHITECTURE.md:83`.
- **Fund 7 (Bestandsliste): teilweise**, siehe neuer Fund 7.

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen.** Beleg: `crates/tui/src/fountain.rs:130`, `crates/script/src/parse.rs:9`.
- **Fund 2 (Breite von Türen und Fenstern in Ablage und Sitzung): offen.** Beleg: `crates/store/src/lib.rs:405`, `crates/services/src/plan_edit/openings.rs:28`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe): offen.** Beleg: `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen.** Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:318`, `crates/services/src/plan_edit/template.rs:142`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach geschrieben): offen.** Beleg: `crates/services/src/plan_edit/group.rs:186`, `crates/services/src/project_edit/templates.rs:141`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): offen.** Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/project_edit/templates.rs:11`.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): offen und gewachsen.** #135 hat ODS, XLSX und Einzelbilder ins Menü gebracht. Seitdem hält auch der Ausgabedialog eine eigene Liste, welche Formate nach Papier fragen. Die App kennt zwei Papiergrößen, die Kommandozeile drei. Die Zeile „neues Ausgabeformat“ in `ARCHITECTURE.md` nennt den Dialog nicht, obwohl #135 ihn ändern musste. Beleg: `crates/app/qml/ExportDialog.qml:23`, `crates/app/src/bridge/export.rs:50`, `crates/app/src/bridge/export.rs:91`, `crates/cli/src/main.rs:200`, `crates/cli/src/main.rs:219`, `ARCHITECTURE.md:85`.
- **Fund 2 (Verbindungsdatei knapp unter der Grenze, ein Aufruf je Mark-Feld): offen.** `bridge.rs` hat 791 Zeilen. Beleg: `crates/app/src/bridge.rs:105`.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 5 (Winkel beim Messen in der App berechnet): offen.** Mit dem Tippen der Wandlänge ist eine zweite Rechnung dieser Art dazugekommen, siehe neuer Fund 3. Beleg: `crates/app/src/tools.rs:472`, `crates/model/src/geometry.rs:44`.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): Rest offen.** Die Kommandozeile schreibt für „keine solche Szene“ und „Szene ohne Plan“ weiter eigene englische Sätze. Beleg: `crates/cli/src/main.rs:173`, `crates/cli/src/main.rs:184`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** Beleg: `crates/app/qml/Workspace.qml`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen.** `bridge/view.rs` ist von 764 auf 777 Zeilen gewachsen. Beleg: `crates/app/src/bridge/view.rs`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert. Beleg: `crates/export/src/layout.rs`, `crates/export/src/scan.rs`.

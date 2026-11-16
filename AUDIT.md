# Code-Audit Klappe, 16.11.2026, Stand 18eb4a1

## Urteil
Das Repo bleibt wartbar, ist aber an einer Stelle teurer geworden. Seit dem achten Audit sind zehn PRs gemergt worden. Fünf davon fügen je eine neue Art Griff hinzu: Notizbreite, Türenden, Lampenstrahl und Lampenrichtung, Breite und Tiefe von Möbeln, Punkte einer Fläche. Ein Griff ist der kleine Punkt, an dem man im Plan zieht, um ein Maß zu ändern. Jede neue Art berührt rund 14 Dateien in vier Paketen, und das meiste davon sind wiederholte Aufzählungen. `ARCHITECTURE.md` hat für diesen Fall keine Zeile (Fund 1).

Besser geworden ist:
- Die Schlüssel der Planformen haben eine eigene Datei bekommen. Der Plan der App ist dadurch von 800 auf 589 Zeilen geschrumpft.
- Die Zielwahl beim Ziehen (`target`) ist von 113 auf 86 Zeilen geschrumpft.
- Die Papiergrößen stehen jetzt einmal im Modell.
- Die Mindestbreite der Notiz zeigt, wie eine Regel an einer Stelle aussieht: Laden, Ziehen und Zeichnen benutzen sie gemeinsam.

Von den offenen Funden des siebten und achten Audits ist keiner behoben.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 365 | 384 (ohne die Kopie von Kante, gezählt wie die Strukturprüfung) |
| Einträge in der Bestandsliste | 32 | 34. Neu sind die beiden flachen Listen der Schlüssel in `plan/key.rs` (`key`, `name`, je 17 Zweige, einer je Elementart). Gewachsen sind die Liste der Aufrufe `switched` (650 auf 655 Zeilen), die Fehlerübersetzung `error_text` (25 auf 26 Zweige), die Datei der Planansicht `display.rs` und die Tests der Ablage um eine Zeile. Geschrumpft sind `target` (113 auf 86 Zeilen) und `copy` (86 auf 81). |
| größte Datei (Zeilen) | 1447 (`crates/stellprobe/src/display.rs`) | 1483 (dieselbe Datei; gewachsen mit den Griffen) |
| längste Funktion (Zeilen) | 138 (`apply` in `plan_edit.rs`) | 138 (dieselbe). `switched` mit 655 Zeilen ist weiter keine Funktion, sondern die Liste der Aufrufe. |

## Funde

### 1. Jede neue Art Griff muss an rund 14 Stellen in vier Paketen nachgetragen werden, und `ARCHITECTURE.md` sagt nicht, wo
Der PR für die Griffe an Möbeln (#152) hat 14 Dateien des Programms geändert. Bei Lampe (#151) und Tür (#148) war es genauso. Neuer Inhalt steht davon nur in zwei Dateien: wo der Griff sitzt (`handles.rs`) und was das Ziehen ändert (`moving.rs`). Der Rest sind Aufzählungen, in die jede Griffart von Hand eingetragen wird.

**Fünf Rollen, die gleich aussehen.** Jede Griffart hat eine eigene Rolle für das Aussehen, sodass es fünf gibt: Zeichnung, Notiz, Öffnung, Lampe, Möbel. Alle sehen gleich aus (Akzentfarbe, gefüllt). Trotzdem wird jede einzeln aufgezählt:
- im Aussehen (`look.rs`),
- viermal in der Übersetzung für den Druck, obwohl Griffe nie gedruckt werden,
- in der Zuordnung zu den Ebenen,
- in zwei Tests.

**Die Frage „zu welchem Element gehört dieser Griff“ ist dreimal beantwortet:**
- in der App, damit ein Klick auf den Griff das Möbel auswählt,
- in der Sitzung, für die Ebene beim Sperren,
- in der Zuordnung der Rollen zu Ebenen.

**Dazu die Leerstellen.** Entfernen, Tooltip, Drehen und Kippen zählen jede Griffart noch einmal auf, nur um zu sagen, dass dort nichts passiert. Jede Griffart hat außerdem die flachen Listen der Schlüssel und die Datei `display.rs` vergrößert. Die Bestandsliste musste deshalb in #152 dreimal angehoben werden.

Wer den nächsten Griff baut, etwa für die Länge einer Wand oder den Radius eines Bogens, findet keine Liste dieser Stellen. Die Zeile „neue Art Element“ in `ARCHITECTURE.md` passt nur halb: Die Schlüsseldatei `plan/key.rs` und das Entfernen nennt sie gar nicht, das Ziehen in `moving.rs` nur ungefähr als „move“. Vergisst man eine Stelle, ist das meist ein Übersetzungsfehler und fällt auf. Vergisst man die Ebene oder die Auswahl, fällt es nicht auf: Der Griff ließe sich auf einer gesperrten Ebene ziehen, oder ein Klick würde nur den Griff wählen. Nach der Regel der Datei selbst („mehr Stellen als aufgeführt ist ein Fund“) ist das ein Fund.

Vorschlag: Stellprobe beschreibt einen Griff einmal, mit einer Rolle für alle gleich aussehenden Griffe und der Angabe, zu welchem Element er gehört. Ebene, Auswahl, Tooltip und Entfernen lesen diese Angabe, statt jede Griffart aufzuzählen. `ARCHITECTURE.md` bekommt eine Zeile „neuer Griff“ mit den Stellen, die dann noch bleiben. Aufwand mittel. Risiko gering: Jede Griffart hat Tests in Stellprobe und in der Sitzung (`object_grips`, `light_grips`, `opening_width`, `note_width`, `drawing_edges`, `drawing_points`; die vier ersten sind in diesem Audit grün gelaufen). Dazu kommen die Tests der Auswahl in `app/src/selection.rs`.

Beleg: `ARCHITECTURE.md:75`, `ARCHITECTURE.md:81`, `crates/stellprobe/src/display.rs:108`, `crates/stellprobe/src/display.rs:118`, `crates/stellprobe/src/display.rs:176`, `crates/stellprobe/src/look.rs:84`, `crates/export/src/style.rs:36`, `crates/export/src/style.rs:63`, `crates/export/src/style.rs:104`, `crates/export/src/style.rs:145`, `crates/services/src/session/layers.rs:121`, `crates/services/src/plan_edit.rs:401`, `crates/services/src/plan_edit.rs:432`, `crates/app/src/selection.rs:46`, `crates/services/src/plan_edit/remove.rs:82`, `crates/app/src/bridge/hints.rs:43`, `crates/services/src/plan_edit/moving.rs:49`, `crates/services/src/plan_edit/moving.rs:505`, `crates/services/src/plan_edit/moving.rs:626`, `crates/services/src/plan_edit/moving.rs:651`, `crates/app/src/plan/key.rs:7`, `crates/app/src/plan/key.rs:49`, `crates/app/src/plan/key.rs:76`, Commit `3b5336b`.

### 2. Ziehen und Tippen desselben Maßes folgen verschiedenen Untergrenzen
Beim Ziehen haben die neuen Griffe eigene Untergrenzen, die beim Tippen nicht gelten:
- Eine Tür wird beim Ziehen nie schmaler als 30 cm. Getippt oder beim Laden gilt jede Breite über null.
- Ein Möbel wird beim Ziehen nie schmaler als 5 cm, getippt ist 1 cm erlaubt.
- Ein Lampenstrahl wird beim Ziehen nie enger als 1°, getippt ist jeder Wert über 0° erlaubt.

Die Untergrenze von 5 cm für Möbel steht außerdem ein zweites Mal als eigene Zahl in der App, für das Höhenwerkzeug (Fund 3 des achten Audits).

Die Folge: Eine Tür, die mit 10 cm eingetippt wurde, springt beim ersten Anfassen ihres Griffs auf 30 cm. Wer eine Grenze ändern will, muss sie an zwei oder drei Stellen suchen. Wie es richtig geht, zeigt die Notiz: Ihre Mindestbreite steht einmal im Modell, und Laden, Ziehen und Zeichnen benutzen sie (#146).

Vorschlag: Die Untergrenzen von Öffnung, Möbelmaß, Höhe und Strahl kommen als Konstanten ins Modell, neben die Prüfungen, die es dort schon gibt (`is_object_size_m`, `is_beam_deg`). Tippen, Ziehen und Laden prüfen dann gegen dieselbe Zahl. Aufwand klein. Risiko gering: `services/tests/opening_width.rs`, `object_grips.rs`, `light_grips.rs`, `note_width.rs` und `store/tests/note_width.rs` prüfen das Verhalten.

Beleg: `crates/services/src/plan_edit/openings.rs:13`, `crates/services/src/plan_edit/check.rs:199`, `crates/store/src/lib.rs:442`, `crates/services/src/plan_edit/moving.rs:20`, `crates/services/src/plan_edit/check.rs:211`, `crates/model/src/library.rs:12`, `crates/services/src/plan_edit/moving.rs:420`, `crates/services/src/light_edit.rs:52`, `crates/model/src/light_preset.rs:9`, `crates/app/src/tools.rs:27`, `crates/model/src/project.rs:209`, `crates/store/src/lib.rs:391`.

### 3. Einen Punkt mit dem Möbel mitdrehen steht fünfmal da, obwohl das Modell es kann
Wenn ein Möbel gedreht ist, muss man ausrechnen, wo eine Stelle auf ihm im Plan liegt, etwa eine Ecke oder ein Griff. Diese Rechnung steht an fünf Stellen jeweils von Hand:
- für den Umriss im Plan,
- für den Körper in 3D,
- für den Platz der Griffe in Stellprobe,
- zweimal in der Sitzung, für die Kantenmitte und das Strecken.

Das Modell hat genau diese Rechnung schon (`Point::turned`), und die Zeichnungen benutzen sie. Soll sich die Drehrichtung oder der Drehpunkt eines Möbels ändern, muss man fünf Stellen gleich ändern. Vergisst man eine, sitzen Griff und Kante nicht mehr übereinander.

Nebenbei: In `moving.rs` ist die Erklärung des Streckens über die neue Funktion der Kantenmitte gerutscht. Die eine Funktion hat jetzt zwei Erklärungen, die andere keine.

Vorschlag: Ein Möbel bekommt im Modell eine Funktion „Punkt auf mir, in Plankoordinaten“ auf Grundlage von `turned`, wie die Zeichnung sie hat, und alle fünf Stellen rufen sie auf. Aufwand klein. Risiko gering: `stellprobe/tests/object_grips.rs`, `services/tests/object_grips.rs` (auch für gedrehte Möbel) und die Tests von `display/objects.rs` und `scene.rs` decken es ab.

Beleg: `crates/model/src/geometry.rs:29`, `crates/model/src/plan.rs:147`, `crates/stellprobe/src/display/objects.rs:16`, `crates/stellprobe/src/scene.rs:427`, `crates/stellprobe/src/display/handles.rs:120`, `crates/services/src/plan_edit/moving.rs:462`, `crates/services/src/plan_edit/moving.rs:476`, `crates/services/src/plan_edit/moving.rs:458`.

### 4. Welche Ausgabe auf Papier druckt, steht zweimal in der App, und das Merken des Papiers ebenfalls nur dort
Seit #144 merkt sich das Projekt das Papier, auf dem zuletzt gedruckt wurde. Welche Formate überhaupt auf Papier drucken (PDF, PNG, SVG, Einzelbilder), steht jetzt zweimal in der App, mit denselben vier Wörtern:
- im Ausgabedialog, der danach entscheidet, ob er nach dem Papier fragt,
- in der Verbindung zur Sitzung, die danach entscheidet, ob sie das Papier merkt.

Auch die Regel „nach einer gelungenen Ausgabe wird das Papier ins Projekt übernommen“ steht in der Verbindung, nicht in der Sitzung. Die Kommandozeile merkt sich das Papier deshalb nicht. Ein neues Format, das druckt, muss in beide Listen. Fehlt es in der zweiten, fragt der Dialog zwar nach dem Papier, vergisst es aber gleich wieder.

Außerdem zerlegt der Dialog den Papiernamen „a4-portrait“ selbst am Bindestrich, eine Regel, die im Modell schon steht (`PaperSetup::from_name`). Das ist der offene Rest von Fund 1 des vierten Audits.

Vorschlag: Die Sitzung sagt je Format, ob es Papier braucht, und gibt die Liste mit den anderen Listen heraus. Sie übernimmt das Papier beim Ausgeben selbst. Die Ansicht des Projekts gibt Größe und Ausrichtung getrennt heraus. Aufwand klein. Risiko gering: `tst_exportdialog.qml`, der Szenariotest `export_to.qml`, `services/tests/paper.rs` und `cli/tests/export.rs` prüfen das Verhalten.

Beleg: `crates/app/qml/ExportDialog.qml:33`, `crates/app/qml/ExportDialog.qml:44`, `crates/app/src/bridge/export.rs:52`, `crates/app/src/bridge/export.rs:117`, `crates/services/src/session/export.rs:157`, `crates/cli/src/main.rs:191`, `crates/model/src/paper.rs:82`, `ARCHITECTURE.md:85`.

### 5. Bestandsliste: Die Planansicht wächst mit jedem Griff, die Datei der Planänderungen steht genau an der Grenze
Die Einschätzung der letzten Audits gilt weiter. Die Listen der Feldnamen (`name` in `field.rs`), der Aufrufe (`switched`) und jetzt der Schlüssel (`key`, `name` in `plan/key.rs`) sind flache Listen und in Ordnung, wie sie sind. Die Listen der Schlüssel wachsen aber mit jeder Griffart um einen Zweig (Fund 1).

**Besser geworden:**
- Der Plan der App ist von 800 auf 589 Zeilen geschrumpft (#142).
- Die Zielwahl beim Ziehen `target` ist von 113 auf 86 Zeilen geschrumpft.
- Das Kopieren `copy` ist von 86 auf 81 Zeilen geschrumpft.

**An oder über der Grenze:**
- Die Planansicht `display.rs` ist mit den Griffen von 1447 auf 1483 Zeilen gewachsen. Rund 900 Zeilen davon sind Tests in derselben Datei (ab Zeile 586). Das ist eine natürliche Naht.
- Die Datei der Planänderungen `plan_edit.rs` ist von 770 auf genau 800 Zeilen gewachsen, vor allem durch die neuen Ziele des Ziehens (`Target`). Diese Ziele benutzt nur `moving.rs`.
- Knapp darunter liegen `bridge.rs` mit 796 Zeilen (statt 791), `bridge/view.rs` mit 782 (statt 777) und die Tests `services/tests/manage.rs` mit 800 und `library.rs` mit 799.

Vorschlag: Die Tests der Planansicht kommen in eine eigene Datei, wie bei `tools/tests.rs`. Die Ziele des Ziehens ziehen zu `moving.rs` um, wo sie benutzt werden. Aufwand klein. Risiko gering: Es wird nur verschoben.

Beleg: `crates/stellprobe/src/display.rs:586`, `crates/services/src/plan_edit.rs:432`, `crates/services/src/plan_edit.rs:800`, `crates/app/src/plan/key.rs:49`, `crates/app/src/plan/key.rs:76`, `crates/app/src/bridge.rs`, `crates/app/src/bridge/view.rs`, `structure.baseline`.

## Seit dem letzten Audit
Seitdem wurden zehn PRs gemergt (#142, #144 bis #146, #148 bis #153). Aus dem siebten und achten Audit hat der Eigentümer noch keinen Fund zum Beheben gewählt und keinen abgelehnt. Behoben ist keiner. Zwei Funde älterer Audits sind teilweise behoben.

Achtes Audit:
- **Fund 1 (Seite und Scharnier der Tür in eigenen Listen der Karte): offen.** Beleg: `crates/app/qml/OpeningCard.qml:31`, `crates/app/qml/OpeningCard.qml:32`, `crates/app/src/object.rs:131`.
- **Fund 2 (Regeln der eigenen Bibliothek in Ablage und Sitzung, Art `form` beim Laden nicht geprüft): offen.** Beleg: `crates/services/src/project_edit/library.rs:78`, `crates/store/src/library.rs:106`.
- **Fund 3 (Regeln des Höhenwerkzeugs in der App, Höhe als erfundener Planpunkt): offen.** Die Mindesthöhe ist jetzt Teil des neuen Funds 2. Beleg: `crates/app/src/tools.rs:27`, `crates/app/src/bridge/free.rs:230`.
- **Fund 4 (Vorlagen weitergeben unter dem Namen der Bibliothek): offen.** Beleg: `crates/app/src/bridge/library.rs:19`.
- **Fund 5 (Punkt in Fläche zweimal): offen.** Beleg: `crates/model/src/library.rs:141`, `crates/stellprobe/src/hit.rs:110`.
- **Fund 6 (3D-Ansicht kopiert den Bildteil des Kamerabilds, Zoomschritt abgeschrieben): offen.** Beleg: `crates/app/qml/FreeView.qml:46`, `crates/stellprobe/src/eye.rs:72`.
- **Fund 7 (Bestandsliste): teilweise.** Der Plan der App ist unter die Grenze gekommen, die Planänderungen sind an sie herangewachsen, siehe neuer Fund 5.

Siebtes Audit:
- **Fund 1 (App errät, ob ein Ziehen am Weg einen Knick gemacht hat): offen.** Beleg: `crates/app/src/bridge/draw.rs:22`, `crates/services/src/plan_edit/moving.rs:129`.
- **Fund 2 (Feld „eigene Höhe oder Vorgabe“ dreimal): offen.** Beleg: `crates/app/qml/WallCard.qml:48`, `crates/app/qml/OpeningCard.qml:66`, `crates/app/qml/ObjectCard.qml:85`.
- **Fund 3 (Regeln für Türen und Fenster in Ablage und Sitzung, zwei Namen für Brüstung und Sturz): offen.** Beleg: `crates/store/src/lib.rs:437`, `crates/store/src/lib.rs:490`, `crates/services/src/plan_edit/heights.rs:74`.
- **Fund 4 (Kamerabild färbt mit einer Kopie der Farbregeln von Kante): teilweise**, unverändert seit dem achten Audit. Beleg: `crates/app/qml/ViewPalette.qml:13`, `crates/app/src/plan/picture.rs:16`.
- **Fund 5 (Raumhöhe eines neuen Drehorts in Modell und Fenster): offen.** Beleg: `crates/model/src/project.rs:183`, `crates/app/qml/LocationEditor.qml:14`.
- **Fund 6 (Dateinamen und `Hint` als Datenquelle der Karten): offen.** Jede neue Griffart trägt sich dort als „kein Tooltip“ ein. Beleg: `crates/app/src/bridge/hints.rs:43`, `crates/services/src/views.rs:390`.

Sechstes Audit:
- **Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers): offen.** Beleg: `crates/tui/src/fountain.rs:130`, `crates/script/src/parse.rs:9`.
- **Fund 2 (Breite von Türen und Fenstern in Ablage und Sitzung): offen und gewachsen.** Das Ziehen hat eine dritte, eigene Untergrenze dazugebracht, siehe neuer Fund 2. Beleg: `crates/store/src/lib.rs:442`, `crates/services/src/plan_edit/openings.rs:25`.
- **Fund 3 (Spalte „Achsensprung gegen“ in Sitzung und Ausgabe): offen.** Beleg: `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen.** Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:318`, `crates/services/src/plan_edit/template.rs:142`.
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach geschrieben): offen.** Beleg: `crates/services/src/plan_edit/group.rs:186`, `crates/services/src/project_edit/templates.rs:141`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): offen.** Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/project_edit/templates.rs:11`.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): teilweise.** Die Papiergrößen stehen seit #144 einmal im Modell, und App und Kommandozeile bieten dieselben drei an. Offen ist, welche Formate drucken, siehe neuer Fund 4. Beleg: `crates/model/src/paper.rs:14`, `crates/app/qml/ExportDialog.qml:33`, `crates/app/src/bridge/export.rs:117`.
- **Fund 2 (Verbindungsdatei knapp unter der Grenze, ein Aufruf je Mark-Feld): offen.** `bridge.rs` hat 796 Zeilen. Beleg: `crates/app/src/bridge.rs:105`.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 5 (Winkel beim Messen in der App berechnet): offen.** Beleg: `crates/app/src/tools.rs:472`, `crates/model/src/geometry.rs:44`.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): Rest offen.** Die Kommandozeile schreibt für „keine solche Szene“ und „Szene ohne Plan“ weiter eigene englische Sätze. Beleg: `crates/cli/src/main.rs:175`, `crates/cli/src/main.rs:186`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** Beleg: `crates/app/qml/Workspace.qml`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen.** `bridge/view.rs` ist von 777 auf 782 Zeilen gewachsen. Beleg: `crates/app/src/bridge/view.rs`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert. Beleg: `crates/export/src/layout.rs:264`, `crates/export/src/scan.rs:85`.

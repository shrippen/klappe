# Code-Audit Klappe, 15.11.2026, Stand 8ea59ff

## Urteil
Das Repo ist wartbar und in besserem Zustand als beim letzten Audit. Alle sieben Funde von damals sind im Code behoben. `ARCHITECTURE.md` gibt es jetzt, und der Code hält sich an die Schichten, die dort stehen. Die Bestandsliste ist von 36 auf 32 Einträge geschrumpft, und die längste Funktion außerhalb der Tests ist kürzer geworden. Was übrig bleibt, ist klein: Fehlermeldungen erscheinen in den drei Programmen auf drei verschiedene Arten. Die neue Fensteraufteilung baut Drehbuch- und Shot-Spalte je zweimal. Eine Datei der Sitzung erledigt mehrere Aufgaben und liegt knapp unter der Größengrenze.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 213 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung) | 220 |
| Einträge in der Bestandsliste | 36 | 32 |
| größte Datei (Zeilen) | 1540 (`crates/stellprobe/src/display.rs`, davon 866 Zeilen Tests) | 1540 (unverändert dieselbe Datei) |
| längste Funktion (Zeilen) | 130 (`plan` in einem Test der Ablage); ohne Tests 122 (`apply` in `plan_edit.rs` und `refresh` in `bridge/view.rs`) | 130 (unverändert `plan` im Test); ohne Tests 122 (nur noch `apply` in `plan_edit.rs`; `refresh` hat jetzt 36 Zeilen). `switched` mit 455 Zeilen ist weiter keine Funktion, sondern die Liste der Aufrufe der Verbindung zur Oberfläche. |

## Funde

### 1. Fehlermeldungen übersetzt nur das Fenster; Terminalprogramm und Kommandozeile zeigen den inneren englischen Text
Die Sitzung meldet einen Fehler immer gleich, zum Beispiel „Szene nicht gefunden“ oder „Projekt lässt sich nicht speichern“. Welcher Text aus dem Sprachkatalog dazu gehört, steht nur in der App, in einer Funktion mit 23 Verzweigungen. Diese Funktion steht auch in der Bestandsliste. Das Terminalprogramm und die Kommandozeile kennen diese Zuordnung nicht. Sie geben den englischen Text aus, den die Programmierer für die Fehlersuche hinterlegt haben. Ein deutscher Nutzer des Terminalprogramms liest dann zum Beispiel „Speichern fehlgeschlagen: cannot write …“. Die Kommandozeile hat zusätzlich eigene englische Sätze im Code („cannot open …“, „no scene …“). Das widerspricht `ARCHITECTURE.md` („Texte für den Nutzer kommen aus dem Katalog, nie aus dem Code“) und `planung/KONZEPT.md` E-10, wo steht, dass die Texte der Kommandozeile aus demselben Katalog kommen wie die der App. Jede neue Fehlerart muss heute in der App eingetragen werden. In den beiden anderen Programmen erscheint sie dann trotzdem nur englisch.

Vorschlag: Die Zuordnung von Fehler zu Text kommt aus der App in die Sitzung, neben die Sprachwahl, die dort schon liegt. Alle drei Programme zeigen Fehler dann über diese eine Stelle an. Aufwand klein. Risiko gering: Die App hat einen Test, der zu jedem Feldnamen einen Text verlangt, und das Terminalprogramm hat eigene Tests. Für Fehlertexte in der Kommandozeile gibt es noch keinen Test, der kommt zuerst.

Beleg: `crates/app/src/notice.rs:8`, `crates/services/src/catalog.rs:12`, `crates/services/src/error.rs:7`, `crates/tui/src/app.rs:354`, `crates/tui/src/main.rs:62`, `crates/cli/src/main.rs:162`, `crates/cli/src/main.rs:171`, `crates/cli/src/main.rs:186`, `planung/KONZEPT.md:115`, `ARCHITECTURE.md` (Abschnitt Layers, letzter Punkt).

### 2. Die neue Fensteraufteilung baut Drehbuch- und Shot-Spalte je zweimal
Seit D-09 zeigt das Fenster die Spalten für Drehbuch und Shot je nach Fensterbreite auf drei Arten: als Spalte neben dem Plan mit einem Griff zum Ziehen, als Schublade über dem Plan oder als Reiter. Für die Spalte, den Griff und die Schublade steht der Aufbau je zweimal fast wörtlich im Code, einmal für das Drehbuch und einmal für den Shot. Die Teile unterscheiden sich nur im Namen und in der Seite, auf der sie erscheinen. Welche Spalte gemeint ist, wird über die Wörter „script“ und „shot“ entschieden. Diese Wörter stehen auch im Hauptfenster und im Menü „Ansicht“. Zwei der vier Korrekturen nach dem PR mussten deshalb beide Hälften gleich ändern: Die Schubladen beginnen jetzt unter der Knopfreihe, und gemerkte Breiten werden auf das Fenster gekürzt. Bei jeder weiteren Korrektur und bei einer dritten Spalte vervielfacht sich das.

Vorschlag: Je ein kleiner Baustein für „Spalte mit Griff“ und für „Schublade“, der den Namen der Spalte und die Seite bekommt. Die Arbeitsfläche setzt jeden davon zweimal ein. Aufwand klein. Risiko gering: `tst_workspace.qml` prüft alle drei Breitenstufen.

Beleg: `crates/app/qml/Workspace.qml:125`, `crates/app/qml/Workspace.qml:133`, `crates/app/qml/Workspace.qml:221`, `crates/app/qml/Workspace.qml:238`, `crates/app/qml/Workspace.qml:249`, `crates/app/qml/Workspace.qml:270`, `crates/app/qml/Workspace.qml:62`, `crates/app/qml/Main.qml:69`, `crates/app/qml/AppHeader.qml:256`, `crates/app/qml/AppHeader.qml:266`, Commits `08d62bb`, `f125988`.

### 3. Die Datei für Änderungen am Projekt hat drei Aufgaben und liegt knapp unter der Größengrenze
`project_edit.rs` regelt Szenen, Drehorte und Figuren. Dazu kommen der Import in die eigene Objektbibliothek und die Planvarianten (Plan hinzufügen, kopieren, umbenennen, löschen). Die Planvarianten laufen dabei auf einem anderen Weg als der Rest: einzeln aufgerufen statt über die gemeinsame Liste der Projektänderungen. Mit 781 Zeilen, davon nur 32 Zeilen Tests, liegt die Datei knapp unter der Grenze von 800. Seit Anfang November wurde sie in 12 Commits geändert. Die nächste Funktion bringt sie über die Grenze, und wer Planvarianten ändern will, muss sich durch Szenen und Bibliothek lesen.

Vorschlag: Planvarianten und Bibliotheksimport kommen je in eine eigene Datei, so wie `plan_edit/` und `session/` schon nach Themen aufgeteilt sind. Die gemeinsamen Hilfen für neue Kennungen bleiben an einer Stelle. Aufwand klein. Risiko gering: `services/tests/variants.rs`, `views.rs`, `manage.rs` und `library.rs` decken diese Teile ab.

Beleg: `crates/services/src/project_edit.rs:17`, `crates/services/src/project_edit.rs:105`, `crates/services/src/project_edit.rs:341`, `crates/services/src/project_edit.rs:376`, `crates/services/src/project_edit.rs:568`, `crates/services/src/project_edit.rs:737`, `crates/services/src/session/scene.rs:90`, `crates/services/src/session/scene.rs:106`.

### 4. Die angezeigten Werte stehen beim Start und beim Auffrischen weiter je in einer eigenen Liste, die niemand gegen die andere prüft
Fund 5 des letzten Audits ist im Kern behoben: Eine Funktion sammelt jetzt alle Werte aus der Sitzung, und Start und Auffrischen nutzen beide dieses Ergebnis. Danach überträgt aber jede der beiden die Werte noch einmal Zeile für Zeile, in zwei getrennten Listen mit je 26 beziehungsweise 24 Zeilen. Fehlt in einer dieser Listen ein neuer Wert, merkt das weder der Bau noch ein Test. Das Fenster zeigt dann beim Start einen leeren Wert oder nach einer Änderung einen alten. Auch der Test für den Start zählt die Werte von Hand auf. `ARCHITECTURE.md` nennt für einen neuen angezeigten Wert sieben Stellen.

Vorschlag: Beide Listen zerlegen das gesammelte Ergebnis vollständig, ohne Auslassung. Dann bricht der Bau ab, wenn ein Wert fehlt, und die Hälfte der sieben Stellen wird vom Werkzeug geprüft statt von Hand. Aufwand klein. Risiko gering: `the_start_fills_every_value` und `app/tests/controller/shown_values.qml` sind vorhanden.

Beleg: `crates/app/src/bridge/view.rs:103`, `crates/app/src/bridge/view.rs:147`, `crates/app/src/bridge/view.rs:187`, `crates/app/src/bridge/view.rs:357`, `crates/app/src/bridge/view.rs:513`, `ARCHITECTURE.md` (Tabelle „Where new things go“, Zeile „New value shown in the window“).

### 5. Bestandsliste: was sich trennen lässt und was so bleiben kann
**Trennt sich an einer Naht:**
- Die Umsetzung eines Plan-Elements in Druckformen (`item_prims`, 106 Zeilen): Der zweite Teil legt nur fest, wo die Beschriftung steht. Er kann eine eigene Funktion werden.
- Das Abtasten von Flächen für PNG (`scan`, 98 Zeilen): Am Anfang wird der Umriss aller Kanten berechnet. Das kann eine eigene Funktion werden.
- Die Übersetzung der Fehler (`error_text`): Sie zieht mit Fund 1 in die Sitzung um.

**In Ordnung, wie sie sind:**
- Die Verteiler der Planänderungen (`apply` in `plan_edit.rs`, `target` in `moving.rs`, `apply` in `edit.rs`, `remove`, `check`, `copy`): Sie wählen nur je Art der Änderung oder des Elements eine Funktion aus.
- Die Feldnamen (`field.rs`) und die Tasten des Terminalprogramms (`editor_key`) sind lange, aber flache Zuordnungstabellen.
- Die Liste der Aufrufe der Verbindung zur Oberfläche (`switched`) ist keine Funktion.
- Das Entpacken und Schreiben von JPEG, PNG, komprimierten Daten und Schriften im Export folgt jeweils dem Dateiformat Schritt für Schritt. Dasselbe gilt für das Lesen des JPEG-Kopfs (`jpeg_frame`).
- Das Speichern eines Projekts (`save` in der Ablage) ist eine gerade Folge von Schritten. Die Zahl der Verzweigungen kommt nur von den Fehlerprüfungen.
- `display.rs` besteht zu mehr als der Hälfte aus Tests. Die großen Testdateien (`store/tests/store.rs`, `services/tests/plan_edit.rs`, `tst_planarea.qml`) lassen sich bei Bedarf nach Thema teilen, nötig ist es nicht.

Aufwand: zusammen klein. Risiko gering: `export/tests/export.rs` und die Tests in `layout.rs` und `scan.rs` decken beide Stellen ab.

Beleg: `crates/export/src/layout.rs:241`, `crates/export/src/layout.rs:298`, `crates/export/src/scan.rs:85`, `crates/app/src/notice.rs:8`, `crates/services/src/plan_edit.rs:409`, `crates/services/src/plan_edit/moving.rs:9`, `crates/services/src/field.rs:80`, `crates/app/src/bridge.rs:89`, `crates/store/src/lib.rs:133`, `crates/stellprobe/src/display.rs:674`.

## Seit dem letzten Audit
- **Fund 1 (eigene Namenslisten der Oberfläche): behoben.** Ebenen, Lichtarten, Strich und Füllung haben jetzt eine vollständige Liste mit Namen und Rückweg im Modell beziehungsweise in der Sitzung. Die Oberfläche bekommt die Listen von dort, auch die Ebenenliste und die Auswahllisten der Marke. Ein Test legt die Namen fest. Beleg: `crates/model/src/plan.rs`, `crates/app/src/bridge/view.rs:232`, `crates/app/src/bridge/view.rs:272`, `crates/app/qml/LayerPopover.qml:16`, `crates/app/qml/MarkCard.qml:19`.
- **Fund 2 (`ARCHITECTURE.md` fehlt): behoben.** Die Datei beschreibt Bausteine, Schichten, wohin Neues gehört und eine bekannte Abweichung. `agent.md` und `planung/KONZEPT.md` verweisen nur noch darauf. Die Abhängigkeiten in den `Cargo.toml` der Pakete stimmen mit der Datei überein. Beleg: `ARCHITECTURE.md`, `agent.md:19`, `planung/KONZEPT.md:62`.
- **Fund 3 (Farbe und Linienart doppelt): behoben.** Das Aussehen jeder Rolle steht einmal in `stellprobe/look.rs`. Der einzige gewollte Unterschied zwischen Bildschirm und Papier ist dort benannt und durch einen Test festgelegt. App und Export übersetzen nur noch. Beleg: `crates/stellprobe/src/look.rs:56`, `crates/app/src/plan.rs:25`, `crates/export/src/style.rs:142`.
- **Fund 4 (PNG-Kopf doppelt): behoben.** Das Paket für Bildköpfe liest jetzt auch den PNG-Kopf mit Farbart. Ablage und Export nutzen es beide, und die Ablage weist beim Hinzufügen ab, was der PDF-Export nicht drucken kann. Beleg: `crates/imagehead/src/lib.rs:167`, `crates/store/src/images.rs:113`, `crates/export/src/png.rs:13`.
- **Fund 5 (Werte für die Oberfläche doppelt geholt): weitgehend behoben.** Eine Funktion sammelt alle Werte, und `refresh` ist aus der Bestandsliste verschwunden. Die zwei Übertragungslisten bleiben, siehe neuer Fund 4. Beleg: `crates/app/src/bridge/view.rs:147`.
- **Fund 6 (Datei der Zeichenwerkzeuge): behoben.** Auswahl und Felder eines Möbels haben eigene Dateien, und `tools.rs` hat noch 507 statt 786 Zeilen. Beleg: `crates/app/src/selection.rs:1`, `crates/app/src/object.rs:1`.
- **Fund 7 (Bestandsliste): bearbeitet.** `refresh`, die Stilzuordnung des Exports, die Farbzuordnung der App und das Lesen des PNG-Kopfs stehen nicht mehr in der Liste. Die übrige Einschätzung gilt weiter, siehe neuer Fund 5. Beleg: `structure.baseline`.
- Abgelehnt hat der Eigentümer keinen Fund.

# Code-Audit Klappe, 17.11.2026, Stand a05fd92b

## Urteil
Das Repo ist wartbar und besser als beim 15. Audit. Alle sechs Funde des 15. Audits sind behoben, jeder in einem eigenen PR (#231, #233 bis #237). Die Bestandsliste ist gleich geblieben (26 Einträge). Die größte Datei und die längste Funktion haben sich nicht verändert.

Neu sind drei Änderungen am Abgleich mit dem Buch: Im Terminal lässt sich eine Antwort mit Strg+Z zurücknehmen (#238), gleiche Zwillingsszenen werden nach ihrer Reihenfolge zugeordnet (#239), und deutsche Szenenköpfe werden erkannt (#240). Der Kern davon sitzt am richtigen Ort. Zwei neue Stellen stehen aber nicht in `ARCHITECTURE.md`, und an einer davon muss eine Liste von Hand mit dem Abgleich übereinstimmen (Funde 1 und 2). #240 hat drei kleine Doppelungen mitgebracht (Fund 3).

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 558 | 559 (gezählt wie die Strukturprüfung, ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | 26 | 26, unverändert. Seit dem letzten Audit hat kein PR die Liste geändert. |
| größte Datei (Zeilen) | 883 (`crates/export/src/jpeg.rs`) | 883, dieselbe Datei. Die nicht geprüfte `bridge.rs` bleibt bei 1041 Zeilen. |
| längste Funktion (Zeilen) | 125 (Testhilfe `plan` in `store/tests/store.rs`), im Programmcode `apply` mit 120 | unverändert 125 und 120 |

## Funde

### 1. Ob sich das Register geändert hat, prüft eine Liste von Feldern, die von Hand zum Abgleich passen muss
Seit #235 werden die offenen Fragen zum Buch nach jeder Änderung neu ausgerechnet. Das geschieht aber nur, wenn sich das Register geändert hat. Ob es sich geändert hat, entscheidet eine eigene Prüfung. Sie vergleicht sieben der neun Angaben eines Eintrags: Kennung, Art, Nummer, Kopf, Zustand, den gespeicherten Text und die abgelehnten Szenen. Ort und Pläne vergleicht sie nicht, weil der Abgleich sie nicht liest. Heute stimmt das.

Was es kostet: Bekommt ein Eintrag eine neue Angabe, die der Abgleich liest, muss sie auch in diese Prüfung. Fehlt sie dort, veralten die offenen Fragen wieder still, genau wie im Fund 1 des 15. Audits. Weder der Compiler noch die Zeile für den Abgleich in `ARCHITECTURE.md` weisen darauf hin.

Vorschlag: Die Prüfung zerlegt den Eintrag in alle seine Angaben und nennt Ort und Pläne ausdrücklich als „egal“. Dann meldet der Compiler jede neue Angabe, bis jemand entschieden hat, ob sie zählt. Dazu nennt die Zeile in `ARCHITECTURE.md` die Prüfung.

Aufwand klein. Risiko gering: Es gibt `crates/services/tests/book_align.rs` (auch für das Entfernen eines Eintrags) und `crates/services/tests/perf.rs`.

Beleg: `crates/services/src/session/book.rs:144`, `crates/services/src/session/undo.rs:60`, `crates/model/src/project.rs:542`, `ARCHITECTURE.md:84`.

### 2. Das Terminal führt selbst Buch darüber, welche Antworten Strg+Z zurücknehmen kann; `ARCHITECTURE.md` sagt, die Oberflächen zeigen nur an
Im Terminal gibt es zwei Verläufe zum Rückgängigmachen: den des Texteditors und den der Sitzung, in dem die Antworten auf die Fragen zum Buch stehen. Seit #238 entscheidet das Terminal bei Strg+Z und Strg+Y selbst, welcher der beiden dran ist. Dafür merkt es sich bei jeder Antwort, wie viele Schritte der Editor gerade hatte. Diese zwei Listen hält es an vier Stellen von Hand richtig: beim Antworten, bei Rückgängig, bei Wiederholen und beim Speichern von geändertem Text. Die Regeln sind gut getestet.

`ARCHITECTURE.md` sagt in der Zeile für den Abgleich, dass Terminal und Fenster nur die Statuszeile und den Fragedialog zeigen und die gewählte Position zurückgeben. Das Zurücknehmen von Antworten steht dort nicht.

Was es kostet: Wer den Abgleich ändert, findet diese Stelle über die Tabelle nicht. Das Fenster bekommt noch einen eigenen Drehbuch-Editor (DB-23, „Known deviations“). Dann braucht es dieselbe Regel und müsste sie ein zweites Mal bauen. Dazu nimmt das Terminal an, dass jeder Schritt im Verlauf der Sitzung eine Antwort ist. Ein zweiter Schritt der Sitzung im Terminal würde das still brechen.

Vorschlag: Jetzt die Zeile in `ARCHITECTURE.md` um diese Stelle ergänzen. Kommt der Editor im Fenster, wandert die Regel „welcher Verlauf zuerst“ in einen gemeinsamen Baustein, und die Oberflächen fragen sie dort ab. Für die Annahme kann das Terminal den Namen des obersten Schritts prüfen, den die Sitzung schon herausgibt.

Aufwand klein (Text), später mittel. Risiko gering: Es gibt sieben Tests in `crates/tui/src/tests/book.rs` ab Zeile 136.

Beleg: `crates/tui/src/app/book.rs:83`, `crates/tui/src/app/book.rs:101`, `crates/tui/src/app/book.rs:118`, `crates/tui/src/app.rs:108`, `crates/tui/src/app.rs:560`, `crates/tui/src/editor.rs:71`, `crates/services/src/session/undo.rs:134`, `ARCHITECTURE.md:84`, `ARCHITECTURE.md:106`.

### 3. Drei kleine Doppelungen aus #240: deutsche Szenenköpfe und die Sprache eines neuen Projekts
Die deutschen Anfänge von Szenenköpfen („INNEN“, „AUSSEN“, „AUßEN“) stehen im Leser des Drehbuchs zweimal: in der Liste aller Anfänge und noch einmal in einer eigenen Prüfung, ob ein Anfang deutsch ist. Nur deutsche Anfänge müssen großgeschrieben sein, weil „Innen ist es kalt.“ sonst ein Szenenkopf wäre. Ein neuer deutscher Anfang, der nur in die erste Liste kommt, macht gewöhnliche Sätze zu Szenenköpfen, die mit diesem Wort beginnen.

Ob ein Wort großgeschrieben ist, prüft #240 mit einer neuen Funktion. Dieselbe Prüfung gibt es 30 Zeilen weiter schon. Die neue Funktion schreibt die Ausnahme für „ß“ als Zeichen hin, statt die vorhandene Konstante zu nutzen. Damit steht die Regel für „ß“ jetzt dreimal im Repo, zusammen mit der Großschreibung im Terminal (siehe unten, alter Fund).

Ein neues Projekt bekommt die Sprache der Oberfläche als Sprache des Drehbuchs. Die Übersetzung von der einen Sprachangabe in die andere steht beim Anlegen. Eine zweite Übersetzung in die Gegenrichtung steht bei den Seiten des Drehbuchs.

Was es kostet: Eine geänderte Regel für Szenenköpfe oder Großschreibung gilt sonst nur an einer der Stellen. Eine dritte Sprache braucht zwei Übersetzungen.

Vorschlag: Die deutschen Anfänge stehen in einer eigenen Liste, aus der sich die Liste aller Anfänge zusammensetzt. Die neue Prüfung auf Großschreibung entfällt zugunsten der vorhandenen. Die Sprachübersetzung kommt an eine Stelle neben den Typ.

Aufwand klein. Risiko gering: Es gibt die Tests in `crates/script/src/tests.rs` (auch für Prosa mit „Innen“) und `crates/services/tests/manage.rs`.

Beleg: `crates/script/src/parse.rs:39`, `crates/script/src/parse.rs:538`, `crates/script/src/parse.rs:545`, `crates/script/src/parse.rs:573`, `crates/script/src/parse.rs:17`, `crates/tui/src/fountain.rs:174`, `crates/services/src/session/file.rs:87`, `crates/services/src/script_pages.rs:83`.

### 4. Bestandsliste
**Unverändert:** alle 26 Einträge. Kein PR seit dem letzten Audit hat einen Eintrag erhöht oder neu angelegt.

**Geteilt, wie vorgeschlagen:** `script/src/assist.rs` ist durch #236 von 769 auf 546 Zeilen geschrumpft. Der Ablauf von Enter und Tab steht jetzt in `flow.rs` (216 Zeilen).

**Schnell gewachsen:** Die Tests des Abgleichs (`services/tests/book_align.rs`) sind in diesem Abschnitt von 411 auf 708 Zeilen gewachsen. Bei diesem Tempo erreichen sie in zwei bis drei PRs die Grenze. Eine natürliche Naht sind die Tests für Zwillingsszenen (#239). Sie können in eine eigene Datei.

**An der Grenze von 800 Zeilen** (unverändert seit dem letzten Audit): `stellprobe/src/motion.rs` mit 797, `export/tests/export.rs` mit 790, `services/tests/session.rs` und `app/src/bridge/view.rs` mit je 788, `model/src/plan.rs` mit 772, `services/src/plan_edit.rs` mit 763. `PlanArea.qml` ist durch #233 von 778 auf 761 Zeilen geschrumpft.

**In Ordnung:** die flachen Listen (`field.rs` `name`, `plan/key.rs` `key`), die alten Einträge der Ausgabe (`jpeg.rs`, `layout.rs`, `scan.rs`) und `apply`.

Beleg: `structure.baseline`, `crates/services/tests/book_align.rs:571`, `crates/script/src/flow.rs:1`.

## Seit dem letzten Audit
Seit dem 15. Audit wurden zehn PRs gemergt: #231 bis #240.

15. Audit (alle sechs Funde vom Eigentümer zum Beheben gewählt):
- **Fund 1 (offene Fragen veralten beim Entfernen einer Szene): behoben (#235).** Jede Änderung über einen Schritt der Sitzung frischt die Fragen auf, wenn sich das Register geändert hat. Ein Test prüft das Entfernen. Den neuen Rest dieser Lösung beschreibt Fund 1 oben. Beleg: `crates/services/src/session/undo.rs:60`, `crates/services/tests/book_align.rs:426`.
- **Fund 2 (`ARCHITECTURE.md` schickt neue Befehle nach `AppHeader.qml`): behoben (#231).** Beleg: `ARCHITECTURE.md:90`, `ARCHITECTURE.md:93`, `ARCHITECTURE.md:96`.
- **Fund 3 (Tooltip eines Griffs entsteht im Fenster): behoben (#233).** Die Brücke liefert den Text-Schlüssel, und `PlanArea.qml` zeigt ihn nur an. Beleg: `crates/app/src/plan/key.rs:182`, `crates/app/src/bridge/hints.rs:62`, `crates/app/qml/PlanArea.qml:717`.
- **Fund 4 (Fragedialog im Terminal und im Fenster doppelt gebaut): behoben (#237).** Die Sitzung gibt die Antwortzeilen heraus, und die doppelten Texte `tui.book.*` sind weg. Beleg: `crates/services/src/session/book.rs:32`, `crates/tui/src/ui_overlay.rs:209`, `crates/app/qml/BookDialog.qml:28`.
- **Fund 5 (freie Kennung, Dateiname des Drehbuchs): behoben (#234).** Beleg: `crates/services/src/project_edit.rs:567`, `crates/store/src/lib.rs:57`.
- **Fund 6 (Bestandsliste, `assist.rs` teilen): behoben (#236).**

14. Audit, Reste:
- **Fund 2 (auskommentierte Teile, innerhalb einer Zeile verschieden gelesen): im Verhalten behoben (#232).** `Vorher \/* weg */ danach.` liest der Leser jetzt wie der Editor. Die Regel steht weiter zweimal, einmal im Leser und einmal in den Markierungen des Editors. Beleg: `crates/script/src/parse.rs:203`, `crates/script/src/inline.rs:104`.
- **Fund 4 (Papier des Drehbuchs an zwei Stellen übersetzt): offen.** Beleg: `crates/services/src/script_pages.rs:98`, `crates/services/src/script_pages.rs:240`.
- **Fund 5 (Enter und Tab): Rest offen.** Das Terminal übersetzt seine Zeilenarten weiter selbst. `Slot::of` ruft weiter nur ein Test auf. Das Zeichen „)“ steht jetzt in `assist.rs`, `flow.rs` und im Terminal. Beleg: `crates/tui/src/fountain.rs:35`, `crates/script/src/flow.rs:26`, `crates/script/src/assist.rs:526`, `crates/script/src/flow.rs:66`, `crates/script/src/assist.rs:52`, `crates/tui/src/app.rs:34`.

13. Audit:
- **Fund 3 (Karten suchen ihr Element mit kopiertem Block, siebenmal): offen.** Beleg: `crates/app/qml/ImageCard.qml:21`, `crates/app/qml/CameraCard.qml:28`, `crates/app/qml/ObjectCard.qml:30` und die anderen Karten.

12. Audit:
- **Fund 5 (Kopieren über einen gemeinsamen Ablauf): vom Eigentümer nicht gewählt.** Er bleibt stehen und wird nicht neu gemeldet.

Aus dem 10., 8., 7., 6., 5., 4. und 3. Audit sind weiter alle Funde offen, die das 15. Audit dort als offen führt. Beim Fund „Die Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers in das Terminalprogramm“ hat #240 die Prüfung der Anfänge von Szenenköpfen an einer Stelle zusammengelegt. Die Großschreibung selbst schreibt das Terminal aber weiter mit eigenen Regeln. Die Ausnahme für „ß“ steht jetzt an drei Stellen (Fund 3). Beleg: `crates/tui/src/fountain.rs:176`, `crates/tui/src/fountain.rs:200`, `crates/script/src/parse.rs:573`.

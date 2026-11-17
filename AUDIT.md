# Code-Audit Klappe, 17.11.2026, Stand 9ecdea6

## Urteil
Das Repo ist wartbar. Beim Planen ist es besser geworden, das Drehbuch ist die neue Schwachstelle.

Beim Planen haben sich die größten Funktionen verkleinert:
- `apply` hat jetzt 128 statt 141 Zeilen.
- `check`, `copy` und `remove` sind ebenfalls kleiner.
- `editor_key` im Terminal ist aus der Bestandsliste verschwunden.
- Drei Funde des 13. Audits sind sauber behoben.

Mit dem Drehbuch sind rund 75 Dateien dazugekommen: Seiten, Schreibhilfen, Suche, Titelseite, Hervorhebungen, Editor im Terminal und PDF. Dabei sind einige Regeln zweimal entstanden. Zwei dieser Doppelungen verhalten sich schon heute verschieden (Funde 1 und 2). Teile des Schreibablaufs liegen nur im Terminal, obwohl `ARCHITECTURE.md` sie in den gemeinsamen Drehbuch-Baustein legt (Fund 5). Das verteuert den geplanten Editor im Fenster.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 425 | 500 (gezählt wie die Strukturprüfung, ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | 31 | 30. Kein Eintrag ist gewachsen. `editor_key` (Terminal) ist weg. `apply`, `check`, `copy`, `remove` und `store/tests/store.rs` sind kleiner geworden. Ein Wert kann noch schrumpfen (siehe Fund 8). |
| größte Datei (Zeilen) | 1011 (`crates/store/tests/store.rs`) | 972 (dieselbe). Ohne Tests ist es weiter `crates/export/src/jpeg.rs` mit 883 Zeilen. Die nicht geprüfte `bridge.rs` ist von 864 auf 978 Zeilen gewachsen. |
| längste Funktion (Zeilen) | 141 (`apply` in `plan_edit.rs`) | 131 (die Testhilfe `plan` in `store/tests/store.rs`). Im Programmcode ist es `apply` mit 128. |

## Funde

### 1. Die Titelseite wird für das PDF mit einer eigenen Regel gelesen, und ihre sechs Felder stehen an vier Stellen
Der Drehbuch-Baustein hat eine Regel dafür, welcher Eintrag der Titelseite welches Feld ist. Groß- und Kleinschreibung und Leerzeichen zählen dabei nicht. Das Formular im Terminal benutzt diese Regel. Die Sitzung baut das PDF aber mit einer eigenen Suche, bei der Leerzeichen zählen, und schreibt die Feldnamen dafür noch einmal hin.

Die beiden Regeln verhalten sich schon heute verschieden. Ich habe es mit `DraftDate: 1. Mai` geprüft: Das Formular zeigt das Datum unter „Draft date“, das PDF lässt es weg.

Die sechs Felder (Titel, Credit, Autor, Quelle, Datum, Kontakt) stehen außerdem an vier Stellen:
- als Liste im Drehbuch-Baustein,
- als Liste der Beschriftungen im Terminal (deren Reihenfolge nur an der Länge geprüft wird),
- als Feld für Feld ausgeschriebene Titelseite in der Ausgabe,
- in der Zuordnung in der Sitzung.

Was es kostet: Ein neues Feld der Titelseite braucht diese vier Stellen. Die Tabelle „Where new things go“ nennt nur `title.rs`. Eine geänderte Regel für die Namen der Einträge gilt sonst nur im Editor.

Vorschlag: Die Sitzung holt die Felder für das PDF über die Funktion des Drehbuch-Bausteins (`title_fields`). Das Terminal ordnet seine Beschriftungen über die Namen der Felder zu, nicht über ihre Reihenfolge.

Aufwand klein. Risiko gering: Es gibt Tests in `crates/script/src/title.rs`, `crates/services/tests/script_pages.rs` und `crates/export/tests/script.rs`. Ein Test mit `DraftDate:` sollte vor dem Umbau dazukommen.

Beleg: `crates/script/src/title.rs:17`, `crates/script/src/title.rs:36`, `crates/services/src/script_pages.rs:176`, `crates/services/src/script_pages.rs:231`, `crates/services/src/script_pages.rs:247`, `crates/export/src/script.rs:72`, `crates/tui/src/i18n.rs:32`, `crates/tui/src/i18n.rs:234`, `ARCHITECTURE.md:93`.

### 2. Was ein auskommentierter Teil (`/* … */`) ist, wird zweimal bestimmt, und die Zeichen von Fountain stehen dreimal
Ein Drehbuch kann Teile mit `/* … */` auskommentieren. Was dazugehört, bestimmt der Drehbuchleser für den ganzen Text. Die Hervorhebungen, die der Editor einfärbt, bestimmen es ein zweites Mal, Zeile für Zeile, mit eigenen Zeichen-Konstanten.

Die beiden Wege verhalten sich verschieden. Ich habe es geprüft: Steht `/*` innerhalb einer Notiz, etwa `[[a /* b]]`, schneidet der Leser alles bis zum nächsten `*/` heraus, auch über mehrere Zeilen. Diese Teile fehlen dann im Drehbuch und im PDF. Der Editor zeigt dieselben Zeilen als gewöhnlichen Text.

Die Zeichen von Fountain (`.`, `@`, `#`, `^`, `/*`, `*/`, `[[`, `]]`, `\`) stehen außerdem einzeln im Leser, in den Schreibhilfen und in den Hervorhebungen. Im Leser sind sie schon für den ganzen Baustein freigegeben.

Was es kostet: Eine Änderung an den Regeln für Kommentare oder Notizen muss man an zwei Stellen gleich machen, und die beiden lesen schon heute anders.

Vorschlag: Die Hervorhebungen übernehmen, was auskommentiert ist, aus dem Leser (der kennt die Zeilen schon). Die Zeichen stehen nur noch im Leser.

Aufwand mittel. Risiko gering: Es gibt Tests in `crates/script/src/inline.rs:186` und `crates/script/src/tests.rs` und Editortests in `crates/tui/src/tests/marks.rs`.

Beleg: `crates/script/src/parse.rs:7`, `crates/script/src/parse.rs:22`, `crates/script/src/parse.rs:113`, `crates/script/src/inline.rs:4`, `crates/script/src/inline.rs:76`, `crates/script/src/assist.rs:51`, `crates/tui/src/fountain.rs:139`.

### 3. Wie ein gebundenes Etikett sein Element nennt, steht in Sitzung, Ablage und Befehlspalette je für sich
Seit #201 kann eine Beschriftung an ein Objekt, eine Figur oder einen Mark gebunden sein. In der Projektdatei steht das als Text, zum Beispiel `object:tisch` oder `mark:anna:2`.

- Die Sitzung schreibt und liest diesen Text mit eigener Regel.
- Die Ablage prüft ihn beim Laden mit zwei weiteren eigenen Regeln: eine für die Notizen am Drehort, eine für die Beschriftungen im Plan.
- Die Befehlspalette des Fensters entscheidet selbst, welche Elemente ein Etikett bekommen können. Sie liest dafür den Anfang des Auswahl-Schlüssels (`object|figure|mark`).

Das Kontextmenü fragt für jeden Eintrag die Brücke, ob er angeboten wird. Die Palette fragt nicht.

Was es kostet: Sollen auch Kameras oder Lampen ein Etikett bekommen, oder ändert sich die Schreibweise in der Datei, muss man vier Stellen in drei Bausteinen finden. Eine davon ist QML und nach `ARCHITECTURE.md` gar nicht vorgesehen: Die Oberfläche soll nichts selbst entscheiden.

Vorschlag: Ein kleiner Typ „gebunden an“ im Datenmodell, mit Schreiben und Lesen an einer Stelle; Ablage und Sitzung benutzen ihn. Die Brücke liefert „Etikett möglich“ wie die anderen Einträge des Kontextmenüs.

Aufwand klein bis mittel. Risiko gering: Es gibt `crates/services/tests/object_change.rs`, `crates/store/tests/object_fields.rs` und `crates/app/tests/qml/tst_objectcard.qml`.

Beleg: `crates/services/src/plan_edit/bound.rs:55`, `crates/services/src/plan_edit/bound.rs:75`, `crates/store/src/lib.rs:445`, `crates/store/src/lib.rs:531`, `crates/app/qml/ObjectCommands.qml:20`, `crates/app/src/bridge/selection.rs:46`, `crates/model/src/project.rs:221`.

### 4. Die Papiergrößen stehen in der Ausgabe zweimal, die Breite des Drehbuchs zweimal
Für das Papier gibt es heute vier Stellen:
- Das Datenmodell hat zwei Arten Papier: eine für den Plan (A4, A3, Letter) und eine für das Drehbuch (A4, Letter).
- Die Sitzung übersetzt das Drehbuch-Papier an zwei Stellen, einmal für den Seitenumbruch und einmal für das PDF.
- In der Ausgabe stehen die Maße desselben Papiers zweimal: für den Plan in Millimetern, für das Drehbuch in Punkten.
- A3 ist dort für das Drehbuch ausgeschrieben, obwohl es nie ankommt.

Die Breite des Textes von 60 Spalten steht im Drehbuch-Baustein und in der Ausgabe je als eigene Zahl.

Was es kostet: Ein neues Papier (etwa Legal) oder ein korrigiertes Maß muss man in der Ausgabe zweimal ändern. Wird die Textbreite geändert, setzt das PDF die Zeilen an der alten Breite aus.

Vorschlag: Die Ausgabe nimmt für beide Wege eine Tabelle der Papiermaße. Die Textbreite kommt mit dem Blatt aus der Sitzung, statt als zweite Zahl in der Ausgabe zu stehen.

Aufwand klein. Risiko gering: Es gibt `crates/export/tests/script.rs` und `crates/export/tests/export.rs`.

Beleg: `crates/model/src/paper.rs:5`, `crates/model/src/script_setup.rs:29`, `crates/services/src/script_pages.rs:96`, `crates/services/src/script_pages.rs:239`, `crates/export/src/layout.rs:70`, `crates/export/src/script.rs:39`, `crates/export/src/script.rs:104`, `crates/script/src/pages.rs:23`, `crates/export/src/script.rs:26`.

### 5. Teile des Ablaufs von Enter und Tab stehen nur im Terminal, nicht im gemeinsamen Drehbuch-Baustein
`ARCHITECTURE.md` sagt: Die Regeln zum Text, die jeder Editor teilt, „the Enter and Tab flow“ eingeschlossen, liegen im Drehbuch-Baustein. Dort steht, was nach welchem Element kommt. Drei Regeln liegen aber nur im Terminal:
- „Diese Zeile ist noch leer“ (auch `()` zählt als leer),
- „Der Cursor steht am Ende des Elements“ (vor einer schließenden Klammer zählt mit),
- „Enter springt bei Text darunter ans Ende der nächsten Zeile“.

Dazu übersetzt das Terminal selbst, welches Element welche Rolle im Ablauf hat. Die gleichwertige Funktion des Drehbuch-Bausteins (`Slot::of`) rufen nur Tests auf.

Was es kostet: Der Editor im Fenster ist geplant (`ARCHITECTURE.md`, „Known deviations“, DB-23). Er müsste diese Regeln abschreiben, und dann verhalten sich beide Editoren auseinander. Dieselbe Richtung hat der offene Fund 1 des sechsten Audits (Großschreibung beim Tippen).

Vorschlag: Die drei Regeln kommen nach `assist.rs`. Das Terminal ruft sie und `Slot::of` auf.

Aufwand klein bis mittel. Risiko gering: Es gibt `crates/tui/src/tests/flow.rs` (418 Zeilen) und die Tests in `crates/script/src/assist.rs:430`.

Beleg: `ARCHITECTURE.md:26`, `ARCHITECTURE.md:104`, `crates/tui/src/app/compose.rs:25`, `crates/tui/src/app/compose.rs:151`, `crates/tui/src/app/compose.rs:160`, `crates/tui/src/fountain.rs:33`, `crates/tui/src/fountain.rs:236`, `crates/script/src/assist.rs:78`, `crates/script/src/assist.rs:591`.

### 6. Beim Seitenumbruch entstehen „(MORE)“ und „(CONT'D)“ je an zwei Stellen, eine davon mit eigenen Spaltenzahlen
Wird ein Dialog über eine Seite gebrochen, steht unten „(MORE)“ und oben der Name mit „(CONT'D)“. Das passiert auf zwei Wegen, und jeder Weg baut beide Zeilen selbst:
- Schnitt an einem Satzende: Die Spaltenzahl, an der „(MORE)“ steht, ist eine eigene Zahl (22). Sie ist eine Abschrift der Spalte der Namen.
- Schnitt an einer Zeile: Welche Hälfte eines Doppeldialogs eine Zeile ist, errät die Seitenverteilung aus der Spaltenzahl 31. Auch das ist eine Abschrift aus dem Baustein der Blöcke.

Was es kostet: Wer die Spalte der Namen oder die Breite eines Doppeldialogs ändert, muss wissen, dass die Seitenverteilung die Zahlen noch einmal hat. Sonst steht „(MORE)“ an der alten Stelle. Neue Wörter für den Umbruch müssen an beiden Stellen gleich behandelt werden.

Vorschlag: Ein Block sagt selbst, wo sein Name steht und zu welcher Hälfte eine Zeile gehört. Eine Funktion baut „(MORE)“ und „(CONT'D)“ für beide Wege.

Aufwand klein. Risiko gering: Es gibt `crates/script/src/pages/flow_tests.rs` und `crates/script/src/pages/tests.rs`.

Beleg: `crates/script/src/pages/flow.rs:22`, `crates/script/src/pages/flow.rs:24`, `crates/script/src/pages/flow.rs:78`, `crates/script/src/pages/flow.rs:188`, `crates/script/src/pages/flow.rs:193`, `crates/script/src/pages/block.rs:14`, `crates/script/src/pages/block.rs:18`, `crates/script/src/pages/block.rs:364`.

### 7. `ARCHITECTURE.md` nennt für ein neues Drehbuch-Element nicht alle Stellen
Die Zeile „New screenplay element“ nennt drei Stellen nicht:
- `fountain.rs` im Terminal nennt sie zwar, aber das Aussehen jeder Zeilenart wird in `ui.rs` festgelegt.
- Ein Element auf der Seite braucht seine Art in `pages.rs`.
- Die Sitzung entscheidet in `script_pages.rs`, wie diese Art im PDF gesetzt wird (fett oder normal).

Was es kostet: Wer der Tabelle folgt, findet diese drei Stellen erst über die Fehler beim Bauen oder gar nicht.

Vorschlag: Die Zeile ergänzen.

Aufwand klein, nur Text. Kein Risiko.

Beleg: `ARCHITECTURE.md:92`, `crates/tui/src/ui.rs:41`, `crates/script/src/pages.rs:99`, `crates/services/src/script_pages.rs:214`.

### 8. Bestandsliste
**Kann schrumpfen:**
- Die PDF-Funktion `write` in `export/pdf.rs` liegt seit #206 unter 80 Zeilen. Ihr Eintrag (89 Zeilen) kann mit `--update` entfallen.

**Wird bald voll:**
- `AppHeader.qml` ist von 707 auf 799 Zeilen gewachsen. Der nächste Befehl überschreitet die Grenze. Die Datei wurde in vier der zehn PRs geändert, weil jeder Befehl dort seinen Eintrag in der Palette bekommt.
- Die Naht ist schon da: #201 hat die Befehle der Objekte nach `ObjectCommands.qml` gelegt, #203 die Seiten-Ausgaben nach `PageExports.qml`.
- Vorschlag: Die übrigen Befehle der Palette ebenso in eigene Dateien, bevor der nächste dazukommt.

**An der Grenze von 800 Zeilen:**
- `stellprobe/src/scene.rs` mit 800 (vorher 776),
- `tst_appheader.qml` und `tst_plantools.qml` mit je 800 (vorher 738 und 701),
- `services/tests/library.rs` und `manage.rs` mit je 800,
- `tst_planarea.qml` mit 797.

Bei den Testdateien hilft dasselbe Vorgehen wie bei `tst_appheader_objects.qml`: nach Bereichen teilen.

**In Ordnung:**
- die flachen Listen (`field.rs` `name`, `plan/key.rs`),
- die alten Einträge der Ausgabe (`jpeg.rs`, `layout.rs`, `scan.rs`),
- `apply`, das kleiner geworden ist.

Die nicht geprüfte `bridge.rs` wächst weiter (978 Zeilen), wie in `ARCHITECTURE.md` vorgesehen.

Beleg: `structure.baseline`, `crates/app/qml/AppHeader.qml:475`, `crates/app/qml/ObjectCommands.qml:1`.

## Seit dem letzten Audit
Seit dem 13. Audit wurden elf PRs gemergt: #198 bis #207, #209 und #210. #211 (`audit13-klein`) ist noch offen.

13. Audit:
- **Fund 1 (Beispielkopie an der Ablage vorbei beiseitegelegt): behoben (#209).** Die Ablage bietet `set_aside` mit ihrer Regel für den freien Namen an. Beleg: `crates/store/src/folder.rs:86`.
- **Fund 2 (Titelregel zweimal): behoben (#209).** Beleg: Commit `8de4da6`, `crates/services/src/folder.rs`.
- **Fund 3 (Karten suchen ihr Element mit kopiertem Block, siebenmal): offen.** Beleg: `crates/app/qml/ImageCard.qml:16` und die sechs anderen Karten.
- **Fund 4 (ODS und XLSX verschlucken die ZIP-Ablehnung, Grenze zweimal): behoben (#210).** Beleg: `crates/export/src/zip.rs:26`, `crates/export/src/zip.rs:109`.
- **Fund 5 (`ARCHITECTURE.md` nennt das ZIP-Archiv nicht bei `export/`): offen, in #211.** Beleg: `ARCHITECTURE.md:28`.
- **Fund 6 (`rotate` gibt nur an `turned` weiter): offen, in #211.** Beleg: `crates/services/src/plan_edit/moving.rs:521`, `crates/services/src/plan_edit/group.rs:68`.

12. Audit:
- **Fund 5 (Kopieren über einen gemeinsamen Ablauf): vom Eigentümer nicht gewählt.** Bleibt stehen und wird nicht neu gemeldet.

Aus dem 10., 8., 7., 6., 5., 4. und 3. Audit sind weiter alle Funde offen, die das 13. Audit dort als offen oder teilweise führt. An den Belegen hat sich nichts geändert. Das Gewicht hat sich bei zwei Funden verschoben:
- **6. Audit, Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers):** offen. Er wird teurer, sobald der Editor im Fenster kommt (siehe neuer Fund 5). Beleg: `crates/tui/src/fountain.rs:189`, `crates/script/src/parse.rs:399`.
- **8. Audit, Fund 1 (Seite und Scharnier der Tür in eigenen Listen der Karte):** unverändert offen.

Geprüft: Die Tests der Bausteine Drehbuch, Terminal, Ausgabe und Ablage sind in diesem Audit grün gelaufen (`cargo test -p klappe-script -p klappe-tui -p klappe-export -p klappe-store`). Die beiden verschiedenen Verhalten in Fund 1 (`DraftDate:`) und Fund 2 (`/*` in einer Notiz) habe ich mit einem kleinen Programm gegen den Drehbuch-Baustein nachgewiesen. Die Strukturprüfung meldet 500 Dateien, 0 Verstöße und 30 Einträge in der Bestandsliste, davon 1 Wert, der schrumpfen kann.

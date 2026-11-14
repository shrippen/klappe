# Code-Audit Klappe, 14.11.2026, Stand 0d38fc2

## Urteil
Das Repo lässt sich gut warten. Die Aufteilung in Bausteine folgt dem Plan: Jede Schicht ist ein eigenes Crate (ein Rust-Paket), und die Abhängigkeiten in den `Cargo.toml` laufen wie in `agent.md` beschrieben. Zu fast jedem Teil gibt es Tests, die das Verhalten prüfen (20 Testdateien für die Projekt-Sitzung, 13 Oberflächentests).

Die Schwächen liegen an zwei Stellen. Erstens wachsen zwei zentrale Dateien mit jeder neuen Funktion weiter: die Verbindung zur Oberfläche und die Projekt-Sitzung. Zweitens steht dasselbe Wissen an mehreren Orten: Namen von Farben und Haltungen, das Lesen eingetippter Zahlen und das Lesen von JPEG-Köpfen. Außerdem greift die Sitzung an der Ablage vorbei selbst auf Dateien zu. Dies ist das erste Audit, darum gibt es keinen Vergleich.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | | 165 (ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | | 47 (ohne 20 in der Kopie von Kante; `structure.baseline` gibt es noch nicht) |
| größte Datei (Zeilen) | | 2777 (`crates/app/src/bridge.rs`) |
| längste Funktion (Zeilen) | | 123 (`role_style`). Der Eintrag `switched` mit 455 Zeilen ist keine Funktion, sondern die Liste der Aufrufe, die die Qt-Brücke anmeldet. |

## Funde

### 1. Die Namen der Figurenfarben stehen viermal im Code, die Namen von Haltung, Ankunft und Abgang in einer eigenen Tabelle der Oberfläche
Eine neue Figurenfarbe muss heute an vier Stellen gleich geschrieben werden: in der Planansicht der App, in der Zeitleiste der Sitzung, im Export und, über einen dritten Weg, beim Lesen und Schreiben der Projektdatei. Vergisst man eine Stelle, zeigt die App die Farbe an, der Ausdruck aber nicht, oder umgekehrt. Für Haltung, Ankunft, Abgang und Tempo hält die Oberfläche eigene Namenstabellen. Diese Namen sind dieselben, die auch die Projektdatei schreibt. Für die Bildgröße (`ShotSize`) ist das schon richtig gelöst: Das Modell hat dort eine Funktion `name()` mit Test.

Vorschlag: Jede dieser Auswahlarten bekommt im Modell einmal eine Funktion „Name“ und eine Funktion „aus dem Namen“, und alle anderen Stellen nutzen sie. Aufwand klein. Risiko gering, Tests für Projektdatei und Export sind vorhanden.

Beleg: `crates/app/src/plan.rs:75`, `crates/services/src/session.rs:1363`, `crates/export/src/style.rs:27`, `crates/app/src/manage.rs:105`, `crates/app/src/manage.rs:113`, `crates/app/src/tools.rs:680`, `crates/app/src/tools.rs:730`, `crates/app/src/tools.rs:734`, Vorbild `crates/model/src/shot.rs:76`.

### 2. Eingetippte Zahlen werden an sechs Stellen gelesen, jede mit eigenen Regeln
Je nach Feld gelten heute andere Regeln. Im Feld Brennweite der Shotliste ist „85 mm“ erlaubt, auf der Kamerakarte wird dieselbe Eingabe abgewiesen. Bei der Breite eines Drehorts ist „8 m“ erlaubt, bei der Größe eines Möbels nicht.

Auch die Rückmeldung ist verschieden. Eine falsche Breite des Drehorts erzeugt eine Meldung in der Statuszeile. Eine falsche Möbelgröße wird ohne jede Nachricht abgewiesen. Wer eine Regel ändern will, zum Beispiel Einheiten überall erlauben, muss alle sechs Stellen finden. Fünf davon liegen in der Oberfläche, die TUI und die Kommandozeile müssten sie also noch einmal bauen.

Vorschlag: Ein Leser für Zahlen mit Einheit und Bereich gehört in die Sitzung. Er gibt immer dieselbe Fehlerart zurück, und die Oberfläche reicht nur den Text weiter. Aufwand klein bis mittel. Risiko gering, Tests für die einzelnen Leser sind vorhanden.

Beleg: `crates/app/src/tools.rs:641`, `crates/app/src/tools.rs:696`, `crates/app/src/tools.rs:703`, `crates/app/src/camera.rs:35`, `crates/app/src/camera.rs:82`, `crates/app/src/manage.rs:118`, `crates/services/src/edit.rs:107`, stille Abweisung `crates/app/src/bridge.rs:1625`.

### 3. Die Sitzung schreibt und liest Dateien selbst, an der Ablage vorbei
Laut Plan ist allein die Ablage (`store`) für Dateien zuständig. Die Sitzung legt aber das Beispielprojekt selbst an und schreibt die Vorschaubilder der Startseite. Sie prüft selbst, ob ein Ordner leer ist, und schreibt die Exportdateien.

Dafür gibt es drei verschiedene Lösungen für „erst daneben schreiben, dann umbenennen“, obwohl die Ablage dafür schon `write_atomic` hat. Die Exporte schreibt die Sitzung sogar ohne diesen Schutz. Außerdem weiß die Startseite, dass die Ablage ihre Sicherungen in Ordnern versteckt, deren Name mit einem Punkt beginnt. Ändert die Ablage diese Namen, zeigt die Startseite falsche Änderungszeiten, ohne dass ein Test der Ablage das merkt.

Vorschlag: Diese Dateizugriffe wandern als Funktionen in die Ablage, und die Sitzung ruft nur diese auf. Aufwand mittel. Risiko gering, `services/tests/start.rs` und `services/tests/example.rs` sind vorhanden.

Beleg: `crates/services/src/example.rs:31`, `crates/services/src/start.rs:93`, `crates/services/src/start.rs:146`, `crates/services/src/session.rs:1435`, `crates/services/src/session.rs:1453`, `crates/store/src/files.rs:231`, `crates/store/src/lib.rs:44`.

### 4. Der Kopf einer JPEG-Datei wird an drei Stellen gelesen, zweimal fast wörtlich kopiert
Die Ablage liest die Bildgröße, und der Export liest Größe und Farbtiefe. Beide Stellen gehen die Abschnitte der Datei Schritt für Schritt durch, mit kopierten Konstanten und gleicher Schleife. Ein dritter Durchlauf im JPEG-Dekoder des Exports duldet anderen Unsinn zwischen den Abschnitten als die beiden ersten.

Eine Datei kann darum in der Ablage als Bild gelten und im Export abgewiesen werden. Eine Korrektur an einer Stelle erreicht die anderen nicht.

Vorschlag: Nur der Export liest JPEG-Köpfe. Die Sitzung, die über beiden liegt, fragt ihn nach der Bildgröße, und die Ablage kopiert die Bilder nur noch. Wenn die Prüfung beim Hinzufügen in der Ablage bleiben muss, kommt der Kopfleser in ein kleines eigenes Crate unter beiden. Aufwand klein bis mittel. Risiko gering, `store/tests/images.rs` und `export/tests/background.rs` sind vorhanden.

Beleg: `crates/store/src/images.rs:14`, `crates/store/src/images.rs:146`, `crates/export/src/image.rs:11`, `crates/export/src/image.rs:158`, `crates/export/src/jpeg.rs:85`.

### 5. Einen Teil ihrer Ansichten baut die App selbst aus dem rohen Modell, den anderen bekommt sie fertig von der Sitzung
Zeitleiste, Shotliste und Drehbuchzeilen liefert die Sitzung als fertige Ansichten. Die Projektliste, die Tooltips und die Felder einer Marke setzt die App dagegen selbst aus den Daten des Modells zusammen. Sie durchsucht dazu selbst die Shots eines Plans und die Möbel eines Drehorts. Die Suche „Drehort der gewählten Szene“ steht dreimal in der App.

Ändert sich der Aufbau des Modells, muss die App mitgeändert werden, obwohl laut `planung/KONZEPT.md` die Oberfläche nur mit der Sitzung spricht. TUI und Kommandozeile können diese Ansichten außerdem nicht nutzen.

Vorschlag: Die Sitzung liefert auch diese Ansichten als fertige Strukturen, wie in `views.rs`, und die App wandelt sie nur noch in JSON (das Datenformat für die Oberfläche). Aufwand mittel. Risiko mittel: Für die Sitzung gibt es Tests, für `describe` in der App keine.

Beleg: `crates/app/src/manage.rs:27`, `crates/app/src/bridge.rs:2433`, `crates/app/src/bridge.rs:2459`, `crates/app/src/bridge.rs:845`, `crates/app/src/bridge.rs:2427`, `crates/app/src/bridge.rs:2525`, Vorbild `crates/services/src/views.rs:7`.

### 6. Die TUI baut Regeln des Drehbuchformats nach, weil der Drehbuchleser keine Zeilennummern liefert
Der Editor der TUI muss für jede Zeile wissen, was sie ist. Der Leser für das Drehbuchformat Fountain sagt das aber nicht. Darum schneidet die TUI auskommentierte Stellen (`/* … */`) selbst heraus, mit einer zweiten Fassung der Regel aus dem Leser. Dann gibt sie jeden Absatz einzeln an den Leser, mit einem vorangestellten Platzhalter, und zählt die Zeilen aus den Zeilenumbrüchen des Ergebnisses zurück.

Ändert sich eine Regel im Leser, etwa beim Maskieren von `/*`, zeigt der Editor andere Zeilenarten an, als der Leser sieht. Kein Test verbindet beide.

Vorschlag: Der Leser gibt zu jedem Element die Zeile an, in der es beginnt, und die TUI nutzt nur noch diese Angabe. Aufwand mittel. Risiko gering, `script/src/tests.rs` und `tui/src/tests.rs` sind vorhanden.

Beleg: `crates/tui/src/fountain.rs:6`, `crates/tui/src/fountain.rs:139`, `crates/tui/src/fountain.rs:228`, `crates/tui/src/fountain.rs:245`, `crates/script/src/parse.rs:52`.

### 7. Die Verbindung zur Oberfläche ist ein einziger Baustein für alles und wird bei fast jeder dritten Änderung angefasst
`bridge.rs` ist die Datei, über die QML (die Sprache der Oberfläche) die Rust-Seite aufruft. Darin steht ein einziger Baustein mit rund 55 Zuständen und einem Block von 1430 Zeilen. Er regelt Wiedergabe, Zeichenwerkzeuge, Notizen, Messen, Maßstab, Projektverwaltung, zuletzt geöffnete Projekte, Export und Bibliothek. Die Datei wurde in 50 der 183 Commits am Code geändert.

Jede neue Funktion verlängert sie, und zwei gleichzeitige Änderungen stoßen hier fast immer aufeinander. `tools.rs` mischt ebenso Zeichenwerkzeuge, die Felder einer Marke und das Umwandeln von Datei-Adressen.

Vorschlag: Der Block wird nach Themen auf eigene Dateien verteilt. cxx-qt (die Brücke zwischen Qt und Rust) erlaubt mehrere solche Blöcke in verschiedenen Dateien. Die Felder einer Marke kommen aus `tools.rs` in eine eigene Datei, wie es `camera.rs` für die Kamerakarte schon vormacht. Aufwand mittel, mehrere kleine PRs, ohne Änderung am Verhalten. Risiko mittel: Die QML-Tests decken die Oberfläche ab, Unit-Tests für die Brücke gibt es kaum.

Beleg: `crates/app/src/bridge.rs:575`, `crates/app/src/bridge.rs:952`, `crates/app/src/bridge.rs:2386`, `crates/app/src/tools.rs:676`, `crates/app/src/tools.rs:849`, Vorbild `crates/app/src/camera.rs:1`.

### 8. Die Projekt-Sitzung ist ein einziger Typ mit 89 Funktionen
`Session` regelt auf einmal Öffnen, Speichern und Sicherung, Szenen und Pläne, Wiedergabe, Ebenen und Sperren, Shots, Export, Bibliothek, Auswahl im Plan, Rückgängig sowie Zeitleiste und Drehbuchansicht. Die Datei wurde in 44 Commits geändert. Wer etwa nur am Export arbeitet, muss sich durch 1300 Zeilen bewegen, und Änderungen an verschiedenen Themen stoßen hier aufeinander.

Vorschlag: Die Daten bleiben in einem Typ, die Funktionen kommen aber nach Thema in eigene Dateien, zum Beispiel `session/export.rs`, `session/playback.rs` und `session/views.rs`. Aufwand mittel. Risiko gering, 20 Testdateien prüfen die Sitzung von außen.

Beleg: `crates/services/src/session.rs:119`, `crates/services/src/session.rs:147`, `crates/services/src/session.rs:601`, `crates/services/src/session.rs:1166`.

### 9. Zu Umwandlungen in Text fehlt die Rückrichtung an derselben Stelle, und die Dokumentation der Schichten stimmt an einer Stelle nicht
Ein Element im Plan wird in `plan.rs` als Text wie `mark:elisabeth:0` geschrieben. Wieder gelesen wird dieser Text aber in `bridge.rs`, mit einer eigenen Liste der Arten. Eine neue Elementart muss darum an beiden Stellen gleich ergänzt werden. Ebenso stehen die Umwandlung von Pfad in Datei-Adresse, ihre Rückrichtung und eine dritte Fassung in QML an drei verschiedenen Orten.

Außerdem behaupten `agent.md` und `planung/KONZEPT.md`, die Ablage nutze den Drehbuchleser. Sie hängt aber nur am Modell, und ihr Kopfkommentar sagt das auch. Wer die Schichten aus der Doku lernt, lernt sie an dieser Stelle falsch.

Vorschlag: Jede Umwandlung steht mit ihrer Rückrichtung in einer Datei, und die beiden Dokumente werden korrigiert. Aufwand klein. Risiko gering.

Beleg: `crates/app/src/plan.rs:88`, `crates/app/src/bridge.rs:2460`, `crates/app/src/plan.rs:107`, `crates/app/src/tools.rs:849`, `crates/app/qml/StartPage.qml:197`, `agent.md:36`, `planung/KONZEPT.md:81`, `crates/store/src/lib.rs:11`, `crates/store/Cargo.toml:9`.

### 10. Bestandsliste: was sich an einer natürlichen Naht trennt und was so bleiben kann
Es gibt noch keine `structure.baseline`. Beim Anlegen sollten `crates/app/qml/Kante` und `crates/export/kante` ausgenommen werden, denn sie stellen 20 der 67 gemeldeten Einträge und sind Kopien aus Kante.

**Trennen sich an einer Naht:**
- `bridge.rs` und `session.rs`: siehe Funde 7 und 8.
- `plan_edit.rs` (1972 Zeilen ohne Tests): Hier stehen Hinzufügen, Entfernen, Markenzeiten mit Tempo und Hintergrundbild nebeneinander. Die Datei `group.rs` zeigt, wie eine solche Abtrennung schon einmal gelungen ist. Die Berechnung von Gehtempo und Weggeschwindigkeit gehört außerdem eher zur Bewegungsrechnung in `stellprobe` als in die Sitzung.

**In Ordnung, wie sie sind:**
- `role_style`, `Field::name` und `error_text`: lange, aber flache Zuordnungstabellen.
- `switched`: die Deklarationsliste der Brücke, keine Funktion.
- Die Dekoder für JPEG, PNG und Deflate im Export: Sie folgen dem jeweiligen Dateiformat Schritt für Schritt.
- `display.rs`: Mehr als die Hälfte der Datei sind Tests.
- Die großen Testdateien (`store/tests/store.rs`, `services/tests/plan_edit.rs`, `tst_planarea.qml`): Man kann sie bei Bedarf nach Thema teilen, nötig ist es nicht.

Aufwand für `plan_edit.rs`: mittel. Risiko gering, `services/tests/plan_edit.rs` und weitere Testdateien sind vorhanden.

Beleg: `crates/services/src/plan_edit.rs:488`, `crates/services/src/plan_edit.rs:1594`, `crates/services/src/plan_edit.rs:1692`, `crates/services/src/plan_edit.rs:269`, `crates/services/src/plan_edit/group.rs:1`, `crates/stellprobe/src/motion.rs:198`, `crates/export/src/style.rs:39`, `crates/services/src/field.rs:80`, `crates/app/src/notice.rs:20`, `crates/app/src/bridge.rs:89`.

## Seit dem letzten Audit
Dies ist das erste Audit, frühere Funde gibt es nicht.

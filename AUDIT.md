# Code-Audit Klappe, 14.11.2026, Stand 0d38fc2

## Urteil
Das Repo lässt sich gut warten: Die Bausteine folgen dem Plan, und zu fast jedem Teil gibt es Tests, die das Verhalten prüfen (20 Testdateien für die Projekt-Sitzung, 12 Oberflächentests). Schwach ist, dass zwei zentrale Dateien mit jeder Funktion weiter wachsen und dasselbe Wissen an mehreren Orten steht. Außerdem greift die Sitzung an der Ablage vorbei selbst auf Dateien zu. Dies ist das erste Audit, darum gibt es keinen Vergleich.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | | 165 (ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | | 47 (ohne 20 in der Kopie von Kante; `structure.baseline` gibt es noch nicht) |
| größte Datei (Zeilen) | | 2777 (`crates/app/src/bridge.rs`) |
| längste Funktion (Zeilen) | | 130 (`plan` in einem Test der Ablage); ohne Tests 123 (`role_style`). Der Eintrag `switched` mit 455 Zeilen ist keine Funktion, sondern die Liste der Aufrufe, die die Verbindung zur Oberfläche anmeldet. |

## Funde

### 1. Die Namen der Figurenfarben stehen viermal im Code, die Namen von Haltung, Ankunft und Abgang in einer eigenen Tabelle der Oberfläche
Eine neue Figurenfarbe muss heute an vier Stellen gleich geschrieben werden: in der Planansicht der App, in der Zeitleiste der Sitzung, im Export und, über einen dritten Weg, beim Lesen und Schreiben der Projektdatei. Drei dieser Stellen werden von Hand gepflegt, die vierte leitet den Namen selbst ab, und alle müssen gleich lauten. Vergessen kann man keine Stelle, denn das Programm lässt sich erst wieder bauen, wenn alle ergänzt sind. Ein Tippfehler im Namen an einer Stelle fällt aber nicht auf: Dann zeigt die App die Farbe an, der Ausdruck nicht, oder umgekehrt. Für Haltung, Ankunft, Abgang und Tempo hält die Oberfläche eigene Namenstabellen; dort fällt auch eine fehlende Zeile nicht auf. Diese Namen sind dieselben, die auch die Projektdatei schreibt. Für die Bildgröße ist das schon richtig gelöst: Das Modell kennt dort den Namen selbst, mit Test.

Vorschlag: Jede dieser Auswahlarten bekommt im Modell einmal eine Funktion „Name“ und eine Funktion „aus dem Namen“, und alle anderen Stellen nutzen sie. Aufwand klein. Risiko gering, Tests für Projektdatei und Export sind vorhanden.

Beleg: `crates/app/src/plan.rs:75`, `crates/services/src/session.rs:1363`, `crates/export/src/style.rs:27`, `crates/app/src/manage.rs:105`, `crates/app/src/manage.rs:113`, `crates/app/src/tools.rs:680`, `crates/app/src/tools.rs:730`, `crates/app/src/tools.rs:734`, Vorbild `crates/model/src/shot.rs:76`.

### 2. Eingetippte Zahlen werden an sechs Stellen gelesen, jede mit eigenen Regeln
Je nach Feld gelten heute andere Regeln. Im Feld Brennweite der Shotliste ist „85 mm“ erlaubt, auf der Kamerakarte wird dieselbe Eingabe abgewiesen. Bei der Breite eines Drehorts ist „8 m“ erlaubt, bei der Größe eines Möbels nicht.

Auch die Rückmeldung ist verschieden. Eine falsche Breite des Drehorts erzeugt eine Meldung in der Statuszeile. Bei einer falschen Möbelgröße wird das Feld nur rot umrandet, ohne Text. Wer eine Regel ändern will, zum Beispiel Einheiten überall erlauben, muss alle sechs Stellen finden. Fünf davon liegen in der Oberfläche; das Terminalprogramm und die Kommandozeile müssten sie also noch einmal bauen.

Vorschlag: Ein Leser für Zahlen mit Einheit und Bereich gehört in die Sitzung. Er gibt immer dieselbe Fehlerart zurück, und die Oberfläche reicht nur den Text weiter. Aufwand klein bis mittel. Risiko gering, Tests für die einzelnen Leser sind vorhanden.

Beleg: `crates/app/src/tools.rs:641`, `crates/app/src/tools.rs:696`, `crates/app/src/tools.rs:703`, `crates/app/src/camera.rs:35`, `crates/app/src/camera.rs:82`, `crates/app/src/manage.rs:118`, `crates/services/src/edit.rs:107`, stille Abweisung `crates/app/src/bridge.rs:1625`.

### 3. Die Sitzung schreibt und liest Dateien selbst, an der Ablage vorbei
Laut Plan ist allein die Ablage (`store`) für Dateien zuständig. Die Sitzung legt aber das Beispielprojekt selbst an und schreibt die Vorschaubilder der Startseite. Sie prüft selbst, ob ein Ordner leer ist, und schreibt die Exportdateien.

Dafür gibt es drei verschiedene Lösungen für „erst daneben schreiben, dann umbenennen“, obwohl die Ablage dafür schon eine eigene Funktion hat. Die Exporte schreibt die Sitzung sogar ohne diesen Schutz. Außerdem weiß die Startseite, dass die Ablage ihre Sicherungen in Ordnern versteckt, deren Name mit einem Punkt beginnt. Ändert die Ablage diese Namen, zeigt die Startseite falsche Änderungszeiten, ohne dass ein Test der Ablage das merkt.

Vorschlag: Diese Dateizugriffe wandern als Funktionen in die Ablage, und die Sitzung ruft nur diese auf. Aufwand mittel. Risiko gering, `services/tests/start.rs` und `services/tests/example.rs` sind vorhanden.

Beleg: `crates/services/src/example.rs:31`, `crates/services/src/start.rs:93`, `crates/services/src/start.rs:146`, `crates/services/src/session.rs:1435`, `crates/services/src/session.rs:1453`, `crates/store/src/files.rs:231`, `crates/store/src/lib.rs:44`.

### 4. Der Kopf einer JPEG-Datei wird an drei Stellen gelesen, zweimal fast wörtlich kopiert
Die Ablage liest die Bildgröße, und der Export liest Größe und Farbtiefe. Beide Stellen gehen die Abschnitte der Datei Schritt für Schritt durch, mit kopierten Konstanten und gleicher Schleife. Ein dritter Durchlauf im Teil des Exports, der JPEG-Bilder entpackt, duldet anderen Unsinn zwischen den Abschnitten als die beiden ersten.

Eine Datei kann darum in der Ablage als Bild gelten und im Export abgewiesen werden. Eine Korrektur an einer Stelle erreicht die anderen nicht.

Vorschlag: Nur der Export liest JPEG-Köpfe. Die Sitzung, die über beiden liegt, fragt ihn nach der Bildgröße, und die Ablage kopiert die Bilder nur noch. Wenn die Prüfung beim Hinzufügen in der Ablage bleiben muss, kommt das Lesen des Kopfs in ein kleines eigenes Paket unter beiden. Aufwand klein bis mittel. Risiko gering, `store/tests/images.rs` und `export/tests/background.rs` sind vorhanden.

Beleg: `crates/store/src/images.rs:14`, `crates/store/src/images.rs:146`, `crates/export/src/image.rs:11`, `crates/export/src/image.rs:158`, `crates/export/src/jpeg.rs:85`.

### 5. Einen Teil ihrer Ansichten baut die App selbst aus dem rohen Modell, den anderen bekommt sie fertig von der Sitzung
Zeitleiste, Shotliste und Drehbuchzeilen liefert die Sitzung als fertige Ansichten. Die Projektliste, die Hinweise beim Überfahren mit der Maus und die Felder einer Marke setzt die App dagegen selbst aus den Daten des Modells zusammen. Sie durchsucht dazu selbst die Shots eines Plans und die Möbel eines Drehorts. Die Suche „Drehort der gewählten Szene“ steht dreimal in der App.

Ändert sich der Aufbau des Modells, muss die App mitgeändert werden, obwohl laut `planung/KONZEPT.md` die Oberfläche nur mit der Sitzung spricht. Terminalprogramm und Kommandozeile können diese Ansichten außerdem nicht nutzen.

Vorschlag: Die Sitzung liefert auch diese Ansichten als fertige Strukturen, wie in `views.rs`, und die App wandelt sie nur noch in JSON (das Datenformat für die Oberfläche). Aufwand mittel. Risiko mittel: Für die Sitzung gibt es Tests, für die Hinweistexte in der App keine.

Beleg: `crates/app/src/manage.rs:27`, `crates/app/src/bridge.rs:2433`, `crates/app/src/bridge.rs:2459`, `crates/app/src/bridge.rs:845`, `crates/app/src/bridge.rs:2427`, `crates/app/src/bridge.rs:2525`, Vorbild `crates/services/src/views.rs:7`.

### 6. Das Terminalprogramm (TUI) baut Regeln des Drehbuchformats nach, weil der Drehbuchleser keine Zeilennummern liefert
Der Editor im Terminal muss für jede Zeile wissen, was sie ist. Der Leser für das Drehbuchformat Fountain sagt das aber nicht. Darum schneidet das Terminalprogramm auskommentierte Stellen (`/* … */`) selbst heraus, mit einer zweiten Fassung der Regel aus dem Leser. Dann gibt es jeden Absatz einzeln an den Leser, mit einem vorangestellten Platzhalter, und zählt die Zeilen aus den Zeilenumbrüchen des Ergebnisses zurück.

Ändert sich eine Regel im Leser, etwa wie ein `/*` im Text als gewöhnliches Zeichen gilt, zeigt der Editor andere Zeilenarten an, als der Leser sieht. Kein Test verbindet beide.

Vorschlag: Der Leser gibt zu jedem Element die Zeile an, in der es beginnt, und das Terminalprogramm nutzt nur noch diese Angabe. Aufwand mittel. Risiko gering, `script/src/tests.rs` und `tui/src/tests.rs` sind vorhanden.

Beleg: `crates/tui/src/fountain.rs:6`, `crates/tui/src/fountain.rs:139`, `crates/tui/src/fountain.rs:228`, `crates/tui/src/fountain.rs:245`, `crates/script/src/parse.rs:52`.

### 7. Die Verbindung zur Oberfläche ist ein einziger Baustein für alles und wird bei fast jeder dritten Änderung angefasst
Über eine einzige Datei ruft die Oberfläche die Programmlogik auf. Darin steht ein einziger Baustein mit 51 Zuständen und einem Block von 1430 Zeilen. Er regelt Wiedergabe, Zeichenwerkzeuge, Notizen, Messen, Maßstab, Projektverwaltung, zuletzt geöffnete Projekte, Export und Bibliothek. Die Datei wurde in 50 der 183 Commits am Code geändert.

Jede neue Funktion verlängert sie, und zwei gleichzeitige Änderungen stoßen hier fast immer aufeinander. Die Datei der Werkzeuge mischt ebenso Zeichenwerkzeuge, die Felder einer Marke und das Umwandeln von Datei-Adressen.

Vorschlag: Der Block wird nach Themen auf eigene Dateien verteilt; die Technik der Verbindung erlaubt mehrere solche Blöcke. Die Felder einer Marke kommen in eine eigene Datei, wie es die Kamerakarte schon vormacht. Aufwand mittel, mehrere kleine PRs, ohne Änderung am Verhalten. Risiko mittel: Die Oberflächentests decken die Oberfläche ab, Tests der Verbindung selbst gibt es kaum.

Beleg: `crates/app/src/bridge.rs:575`, `crates/app/src/bridge.rs:952`, `crates/app/src/bridge.rs:2386`, `crates/app/src/tools.rs:680`, `crates/app/src/tools.rs:849`, Vorbild `crates/app/src/camera.rs:1`.

### 8. Die Projekt-Sitzung ist ein einziger Baustein mit 89 Funktionen
Er regelt auf einmal Öffnen, Speichern und Sicherung, Szenen und Pläne, Wiedergabe, Ebenen und Sperren, Shots, Export, Bibliothek, Auswahl im Plan, Rückgängig sowie Zeitleiste und Drehbuchansicht. Die Datei wurde in 44 Commits geändert. Wer etwa nur am Export arbeitet, muss sich durch 1300 Zeilen bewegen, und Änderungen an verschiedenen Themen stoßen hier aufeinander.

Vorschlag: Die Daten bleiben in einem Baustein, die Funktionen kommen aber nach Thema in eigene Dateien (Export, Wiedergabe, Ansichten). Aufwand mittel. Risiko gering, 20 Testdateien prüfen die Sitzung von außen.

Beleg: `crates/services/src/session.rs:119`, `crates/services/src/session.rs:147`, `crates/services/src/session.rs:601`, `crates/services/src/session.rs:1166`.

### 9. Zu Umwandlungen in Text fehlt die Rückrichtung an derselben Stelle, und die Dokumentation der Schichten stimmt an einer Stelle nicht
Ein Element im Plan wird für die Oberfläche als kurzer Text geschrieben (etwa `mark:elisabeth:0` für Marke 0 von Elisabeth). Wieder gelesen wird dieser Text aber in einer anderen Datei, mit einer eigenen Liste der Arten. Eine neue Elementart muss darum an beiden Stellen gleich ergänzt werden. Ebenso stehen die Umwandlung von Dateipfad in Datei-Adresse, ihre Rückrichtung und eine dritte Fassung in der Oberfläche an drei verschiedenen Orten.

Außerdem behaupten `agent.md` und `planung/KONZEPT.md`, die Ablage nutze den Drehbuchleser. Sie hängt aber nur am Modell, und ihr Kopfkommentar sagt das auch. Wer die Schichten aus der Doku lernt, lernt sie an dieser Stelle falsch.

Vorschlag: Jede Umwandlung steht mit ihrer Rückrichtung in einer Datei, und die beiden Dokumente werden korrigiert. Aufwand klein. Risiko gering.

Beleg: `crates/app/src/plan.rs:88`, `crates/app/src/bridge.rs:2460`, `crates/app/src/plan.rs:107`, `crates/app/src/tools.rs:849`, `crates/app/qml/StartPage.qml:197`, `agent.md:36`, `planung/KONZEPT.md:81`, `crates/store/src/lib.rs:11`, `crates/store/Cargo.toml:9`.

### 10. Bestandsliste: was sich an einer natürlichen Naht trennt und was so bleiben kann
Es gibt noch keine `structure.baseline`. Beim Anlegen sollten die Kopien aus Kante ausgenommen werden: Die 20 der 67 gemeldeten Einträge in `crates/app/qml/Kante` stammen von dort.

**Trennen sich an einer Naht:**
- `bridge.rs` und `session.rs`: siehe Funde 7 und 8.
- `plan_edit.rs` (1972 Zeilen ohne Tests): Hier stehen Hinzufügen, Entfernen, Markenzeiten mit Tempo und Hintergrundbild nebeneinander. Die Datei `group.rs` zeigt, wie eine solche Abtrennung schon einmal gelungen ist. Die Berechnung von Gehtempo und Weggeschwindigkeit gehört außerdem eher zur Bewegungsrechnung in `stellprobe` als in die Sitzung.

**In Ordnung, wie sie sind:**
- Die Stilzuordnung des Exports, die Feldnamen und die Fehlertexte: lange, aber flache Zuordnungstabellen.
- Die Liste der Aufrufe der Verbindung zur Oberfläche: keine Funktion.
- Das Entpacken von JPEG, PNG und komprimierten Daten im Export: Es folgt dem jeweiligen Dateiformat Schritt für Schritt.
- `display.rs`: Mehr als die Hälfte der Datei sind Tests.
- Die großen Testdateien (`store/tests/store.rs`, `services/tests/plan_edit.rs`, `tst_planarea.qml`): Man kann sie bei Bedarf nach Thema teilen, nötig ist es nicht.

Aufwand für `plan_edit.rs`: mittel. Risiko gering, `services/tests/plan_edit.rs` und weitere Testdateien sind vorhanden.

Beleg: `crates/services/src/plan_edit.rs:488`, `crates/services/src/plan_edit.rs:1594`, `crates/services/src/plan_edit.rs:1692`, `crates/services/src/plan_edit.rs:269`, `crates/services/src/plan_edit/group.rs:1`, `crates/stellprobe/src/motion.rs:198`, `crates/export/src/style.rs:39`, `crates/services/src/field.rs:80`, `crates/app/src/notice.rs:20`, `crates/app/src/bridge.rs:89`.

## Seit dem letzten Audit
Dies ist das erste Audit, frühere Funde gibt es nicht.

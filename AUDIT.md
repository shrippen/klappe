# Code-Audit Klappe, 14.11.2026, Stand 30f23f5

## Urteil
Das Repo lässt sich besser warten als beim letzten Audit. Alle zehn Funde sind bearbeitet und neun davon im Code behoben. Die beiden großen Dateien sind nach Themen aufgeteilt, die Bestandsliste ist von 47 auf 36 Einträge geschrumpft, und die Strukturprüfung läuft jetzt in der CI. Was bleibt, ist kleiner: Die Oberfläche führt für einige Auswahlarten weiter eigene Namenslisten. Bildschirm und Ausdruck entscheiden je für sich, in welcher Farbe und Linienart ein Element erscheint. Die Beschreibung der Schichten steht in zwei Dokumenten, die von Hand gleich gehalten werden.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 165 (ohne die Kopie von Kante) | 213 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung; ein Teil des Zuwachses kommt vom Aufteilen großer Dateien) |
| Einträge in der Bestandsliste | 47 (ohne die Kopie von Kante) | 36 |
| größte Datei (Zeilen) | 2777 (`crates/app/src/bridge.rs`) | 1540 (`crates/stellprobe/src/display.rs`, davon 866 Zeilen Tests) |
| längste Funktion (Zeilen) | 130 (`plan` in einem Test der Ablage); ohne Tests 123 (`role_style`) | 130 (unverändert `plan` in einem Test der Ablage); ohne Tests 122 (`apply` in `plan_edit.rs` und `refresh` in `bridge/view.rs`). `switched` mit 455 Zeilen ist weiter keine Funktion, sondern die Liste der Aufrufe der Verbindung zur Oberfläche. |

## Funde

### 1. Für Ebenen, Lichtarten, Strich, Füllung und die Auswahl an einer Marke hält die Oberfläche eigene Namenslisten
Fund 1 des letzten Audits hat die Namen der Figurenfarben, der Haltung, der Ankunft und des Abgangs ins Modell gelegt. Das Modell ist der Teil des Programms, der die Projektdaten beschreibt. Für andere Auswahlarten gibt es solche Listen weiter in der Oberfläche.

Die Namen der sechs Ebenen stehen an fünf Stellen: einmal als Umwandlung in Text, einmal getrennt davon als Rückweg, zweimal als feste Felder für die Anzeige von „sichtbar“ und „gesperrt“ und einmal als Liste in der Oberfläche. Kommt eine Ebene hinzu, bricht der Bau des Programms nur an der ersten dieser Stellen ab, und an den vier anderen fehlt sie dann ohne Warnung. Lichtarten, Strich und Füllung der Zeichenwerkzeuge haben in der App eigene Tabellen neben den Namen, die das Modell in die Projektdatei schreibt. Die Auswahllisten für Haltung, Ankunft und Abgang stehen fest in der Karte einer Marke, obwohl das Modell sie inzwischen vollständig kennt.

Vorschlag: Jede dieser Arten bekommt wie `ElementKind` und `Pose` eine vollständige Liste mit „Name“ und „aus dem Namen“, und die Oberfläche bekommt die Listen von dort, wie sie es bei den Farben schon tut. Aufwand klein. Risiko gering, es gibt Tests für Projektdatei und Werkzeugleiste.

Beleg: `crates/app/src/notice.rs:8`, `crates/app/src/bridge/layers.rs:51`, `crates/app/src/bridge/view.rs:189`, `crates/app/src/bridge/view.rs:203`, `crates/app/qml/LayerPopover.qml:16`, `crates/app/src/tools.rs:17`, `crates/app/src/tools.rs:27`, `crates/app/src/tools.rs:34`, `crates/app/qml/MarkCard.qml:19`, Vorbild `crates/app/src/plan.rs:91`, `crates/model/src/plan.rs:264`, `crates/app/src/bridge/view.rs:173`.

### 2. Es gibt keine `ARCHITECTURE.md`; der Aufbau steht doppelt in `agent.md` und `planung/KONZEPT.md`
`ARCHITECTURE.md` ist die Datei, die nach den Regeln aller Repos den gewollten Aufbau festhält. Sie fehlt. Stattdessen beschreiben `agent.md` und `planung/KONZEPT.md` die Schichten mit fast derselben Zeichnung und denselben Sätzen. Seit dem letzten Audit mussten vier Commits beide Dateien gleich korrigieren. Eine Stelle, die sagt, wo eine neue Art von Änderung hingehört (neue Ebene, neues Exportformat, neues Feld), gibt es nicht. Ohne diese Datei hat das Audit keinen Maßstab für die Schichten, an dem es messen könnte.

Vorschlag: Die Hauptsitzung legt nach dem Skill `architektur` eine `ARCHITECTURE.md` an. Die Zeichnung der Schichten zieht aus `agent.md` und `KONZEPT.md` dorthin um, und beide verweisen nur noch darauf. Aufwand klein. Risiko keins, es ändert sich kein Code. Der Eigentümer bestätigt die erste Fassung.

Beleg: `agent.md:20`, `planung/KONZEPT.md:60`, Commits `978448f`, `aa9fdc5`, `45a6040`, `3e87a1d`.

### 3. Farbe und Linienart eines Elements legen Bildschirm und Ausdruck je für sich fest
Laut Konzept steht im Ausdruck dasselbe wie am Bildschirm. Welche Farbe und welche Linienart jedes Element hat, steht aber in zwei Tabellen: einer für die Planansicht der App und einer für PDF und SVG. Beide sagen zum Beispiel „Achse orange gestrichelt“ und „Kameraweg gelb gepunktet“, und beide übersetzen die Linienart einer Zeichnung je auf eigene Weise. An einigen Stellen weichen sie schon voneinander ab: Eine Marke ist am Bildschirm nur umrandet, im Ausdruck getönt. Ob das gewollt ist, sagt keine der beiden Stellen. Wer etwa die Achse gepunktet haben will, muss beide Tabellen ändern, und kein Test merkt, wenn eine fehlt.

Vorschlag: Die Darstellungsliste in `stellprobe` gibt zu jeder Rolle einmal Farbe, Linienart und Art der Füllung an. App und Export übersetzen das nur noch in ihr Format (Kante-Name oder Druckfarbe, Strichstärke), und gewollte Unterschiede stehen dort ausdrücklich. Aufwand klein bis mittel. Risiko gering, beide Tabellen haben Tests.

Beleg: `crates/app/src/plan.rs:25`, `crates/app/src/plan.rs:46`, `crates/app/src/plan.rs:47`, `crates/app/src/plan.rs:51`, `crates/app/src/plan.rs:60`, `crates/export/src/style.rs:27`, `crates/export/src/style.rs:64`, `crates/export/src/style.rs:109`, `crates/export/src/style.rs:119`, `crates/export/src/style.rs:89`, `planung/KONZEPT.md:86`.

### 4. Der Kopf einer PNG-Datei wird an zwei Stellen gelesen, mit verschiedenen Regeln
Fund 4 des letzten Audits ist für JPEG gelöst: Es gibt jetzt ein kleines Paket, das den Kopf einer JPEG-Datei liest, und Ablage und Export nutzen es beide. Für PNG fehlt das. Die Ablage nimmt beim Hinzufügen eines Hintergrundbilds jede Datei an, die mit der PNG-Kennung beginnt und an einer festen Stelle zwei Zahlen über null hat. Ob dort wirklich der Kopf der Datei (der Abschnitt `IHDR`) steht und ob die Farbart gültig ist, prüft sie nicht. Der Export prüft beides. Eine Datei mit unbekannter Farbart, zum Beispiel 1, wird darum als Hintergrund angenommen, und der Export bricht später mit „unknown PNG colour type“ ab. Dasselbe gilt für JPEG mit CMYK-Farben, die die Ablage annimmt und der PDF-Export abweist.

Vorschlag: Das Paket für Bildköpfe liest auch den PNG-Kopf, und die Ablage prüft beim Hinzufügen mit denselben Regeln, die der Export braucht. Aufwand klein. Risiko gering, `store/tests/images.rs` und `export/tests/background.rs` sind vorhanden.

Beleg: `crates/store/src/images.rs:117`, `crates/store/src/images.rs:126`, `crates/export/src/png.rs:93`, `crates/export/src/png.rs:118`, `crates/export/src/png.rs:120`, `crates/export/src/image.rs:206`, `crates/imagehead/src/lib.rs:1`.

### 5. Die Werte für die Oberfläche werden zweimal von Hand aus der Sitzung geholt
Beim Start füllt eine Funktion alle Anzeigewerte der Oberfläche (Titel, Plan, Zeitleiste, Shotliste, Ebenen …). Nach jeder Änderung holt eine zweite Funktion dieselben Werte noch einmal mit eigenem Code. Diese zweite Funktion ist mit 122 Zeilen eine der längsten im Repo und steht in der Bestandsliste. Eine neue Anzeige muss darum an fünf Stellen ergänzt werden: in der Anmeldung der Eigenschaft, im Speicherfeld, beim Startwert, beim Füllen und beim Auffrischen. Fehlt sie beim Auffrischen, zeigt die Oberfläche nach einer Änderung einen alten Wert.

Vorschlag: Eine Funktion stellt alle Werte aus der Sitzung zusammen, und Start und Auffrischen nutzen beide dieses Ergebnis. Aufwand klein bis mittel. Risiko mittel: Die Oberflächentests arbeiten mit Attrappen, die Verbindung selbst prüfen nur einige Tests in `bridge/view.rs` und drei Abläufe in `app/tests/controller`.

Beleg: `crates/app/src/bridge/view.rs:103`, `crates/app/src/bridge/view.rs:245`, `crates/app/src/bridge/view.rs:325`, `crates/app/src/bridge.rs:22`, `crates/app/src/bridge.rs:582`, `crates/app/src/bridge.rs:676`.

### 6. Die Datei der Zeichenwerkzeuge hat wieder mehrere Aufgaben und steht kurz vor der Größengrenze
Nachdem die Felder von Marke und Kamera ausgezogen sind, regelt die Datei weiter die Wahl des Werkzeugs, die Auswahl im Plan, Messen, Maßstab, Flächen, Notizen, die Felder eines Möbels (Größe, Höhe, Bibliothek), die Vorschau beim Ziehen und die Tasten. Mit 786 Zeilen liegt sie knapp unter der Grenze von 800. Seit dem letzten Audit wurde sie in 8 von 52 Commits geändert. Die nächste Funktion bringt sie über die Grenze, und Änderungen an verschiedenen Themen stoßen hier aufeinander.

Vorschlag: Die Auswahl und die Felder eines Möbels kommen in eigene Dateien, so wie es `mark.rs` und `camera.rs` schon vormachen. Aufwand klein. Risiko gering, `tools/tests.rs` deckt die Datei ab.

Beleg: `crates/app/src/tools.rs:155`, `crates/app/src/tools.rs:285`, `crates/app/src/tools.rs:640`, `crates/app/src/tools.rs:675`, Vorbild `crates/app/src/mark.rs:1`, `crates/app/src/camera.rs:1`.

### 7. Bestandsliste: was sich trennen lässt und was so bleiben kann
**Trennt sich an einer Naht:**
- Das Auffrischen der Oberfläche (`refresh`, 122 Zeilen): siehe Fund 5.

**In Ordnung, wie sie sind:**
- Die Verteiler der Planänderungen (`apply` in `plan_edit.rs`, `target` in `moving.rs`): Sie wählen nur je Art der Änderung oder des Elements eine Funktion aus.
- Die Fehlertexte, die Feldnamen und die Stilzuordnung des Exports sind lange, aber flache Zuordnungstabellen. Für die Stilzuordnung gilt trotzdem Fund 3.
- Die Liste der Aufrufe der Verbindung zur Oberfläche (`switched`) ist keine Funktion.
- Das Entpacken und Schreiben von JPEG, PNG, komprimierten Daten und Schriften im Export folgt jeweils dem Dateiformat Schritt für Schritt.
- `display.rs` besteht zu mehr als der Hälfte aus Tests.
- Die großen Testdateien (`store/tests/store.rs`, `services/tests/plan_edit.rs`, `tst_planarea.qml`) lassen sich bei Bedarf nach Thema teilen, nötig ist es nicht.

Aufwand: siehe Fund 5. Risiko: siehe dort.

Beleg: `crates/app/src/bridge/view.rs:245`, `crates/services/src/plan_edit.rs:409`, `crates/services/src/plan_edit/moving.rs:9`, `crates/app/src/notice.rs:20`, `crates/services/src/field.rs:80`, `crates/export/src/style.rs:27`, `crates/app/src/bridge.rs:16`, `crates/export/src/jpeg.rs:85`, `crates/stellprobe/src/display.rs:675`.

## Seit dem letzten Audit
- **Fund 1 (Namen der Figurenfarben und Marken): weitgehend behoben.** Farbe, Haltung, Ankunft, Abgang und Tempo haben ihren Namen jetzt einmal im Modell beziehungsweise in der Sitzung, und Tests verknüpfen ihn mit der Projektdatei. Offen sind die Auswahllisten in der Karte einer Marke und dieselbe Art Tabelle für Ebenen, Licht, Strich und Füllung, siehe neuer Fund 1. Beleg: `crates/model/src/project.rs:102`, `crates/model/src/plan.rs:203`, `crates/app/qml/MarkCard.qml:19`.
- **Fund 2 (eingetippte Zahlen): behoben.** Ein Leser in der Sitzung gilt für App und Kommandozeile, und Fehler gehen immer in die Statuszeile. Beleg: `crates/services/src/number.rs:55`, `crates/app/src/bridge/draw.rs:319`, `crates/cli/src/main.rs:223`.
- **Fund 3 (Dateizugriffe an der Ablage vorbei): behoben.** Die Sitzung greift nicht mehr selbst auf Dateien zu. Ein kleiner Rest bleibt: Die Startseite weiß, dass die Ablage halbfertige Dateien mit einem Punkt am Anfang benennt. Ein Test in `services/tests/start.rs` bemerkt es, wenn sich das ändert. Beleg: `crates/store/src/tree.rs:1`, `crates/services/src/start.rs:123`, `crates/store/src/files.rs:12`.
- **Fund 4 (JPEG-Kopf): für JPEG behoben** durch das neue Paket `imagehead`. Der PNG-Kopf ist weiter doppelt, siehe neuer Fund 4. Beleg: `crates/imagehead/src/lib.rs:32`.
- **Fund 5 (Ansichten aus dem rohen Modell): behoben.** Projektliste, Hinweistexte und Drehort liefert die Sitzung fertig. Beleg: `crates/services/src/session/views.rs:36`, `crates/services/src/session/views.rs:84`.
- **Fund 6 (Zeilennummern im Terminalprogramm): behoben.** Der Drehbuchleser gibt zu jedem Element seine Zeilen an, und das Terminalprogramm baut keine Regeln mehr nach. Beleg: `crates/script/src/lib.rs:52`, `crates/tui/src/fountain.rs:113`.
- **Fund 7 (Verbindung zur Oberfläche): behoben.** Sie ist auf 17 Dateien nach Thema verteilt, und die Felder einer Marke haben eine eigene Datei. Es bleibt ein einziger Baustein mit rund 50 Zuständen, wie es die Technik der Verbindung nahelegt. Das doppelte Füllen dieser Zustände ist neuer Fund 5. Beleg: `crates/app/src/bridge.rs:570`, `crates/app/src/mark.rs:1`.
- **Fund 8 (Projekt-Sitzung): behoben.** Sie ist auf 11 Dateien nach Thema verteilt, und `session.rs` hat noch 133 Zeilen. Beleg: `crates/services/src/session.rs:1`.
- **Fund 9 (Umwandlung und Rückrichtung, Doku der Ablage): behoben.** Schlüssel und Datei-Adressen stehen je mit ihrer Rückrichtung an einer Stelle, und die Doku nennt die Abhängigkeiten der Ablage richtig. Für Ebenen fehlt das Muster noch, siehe neuer Fund 1. Beleg: `crates/app/src/plan.rs:126`, `crates/app/src/plan.rs:156`, `crates/app/src/fileurl.rs:9`, `agent.md:37`.
- **Fund 10 (Bestandsliste): behoben.** `structure.baseline` gibt es jetzt, die Kopie von Kante ist ausgenommen, und die CI prüft die Grenzen. `plan_edit.rs` ist nach Thema auf 7 Dateien verteilt. Beleg: `structure.baseline:4`, `crates/services/src/plan_edit/add.rs:1`.
- Abgelehnt hat der Eigentümer keinen Fund.

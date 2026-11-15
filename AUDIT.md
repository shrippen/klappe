# Code-Audit Klappe, 15.11.2026, Stand 8e8dd78

## Urteil
Das Repo ist weiter wartbar. Die Schichten halten: Jedes Paket hängt nur von den Paketen ab, die `ARCHITECTURE.md` erlaubt. Die neuen Teile (Kamera-Marks, Lampen, Notizkarten, Tabellen als ODS und XLSX) stehen dort, wo die Datei sie vorsieht, und haben eigene Tests. Gegenüber dem dritten Audit ist der Zustand etwas schlechter. Von den Funden damals ist nur einer teilweise behoben, weil der Eigentümer noch nicht entschieden hat. Dazu kommt Neues aus den zehn PRs: Die Lampenkarte ist eine Kopie der Kamerakarte. Die Auswahl beim Export (Format, Papier, Reihenfolge) steht in jeder Oberfläche als eigene Liste. Die Verbindungsdatei zwischen Fenster und Sitzung liegt 16 Zeilen unter der Größengrenze.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 220 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung) | 260 |
| Einträge in der Bestandsliste | 32 | 31 |
| größte Datei (Zeilen) | 1540 (`crates/stellprobe/src/display.rs`) | 1534 (dieselbe Datei; der Eintrag in der Bestandsliste kann auf diesen Wert sinken) |
| längste Funktion (Zeilen) | 130 (`plan` in einem Test der Ablage); ohne Tests 122 (`apply` in `plan_edit.rs`) | 130 (unverändert `plan` im Test); ohne Tests 125 (weiter `apply` in `plan_edit.rs`, gewachsen). `switched` ist von 455 auf 496 Zeilen gewachsen. Es ist weiter keine Funktion, sondern die Liste der Aufrufe der Verbindung zur Oberfläche. |

## Funde

### 1. Formate, Papiergrößen und Reihenfolge der Ausgabe stehen in jeder Oberfläche als eigene Liste
Beim Export gibt es drei Auswahlen: das Dateiformat, die Papiergröße und die Reihenfolge der Shotliste (Schnitt oder Dreh). Für keine davon gibt es eine gemeinsame Liste mit Namen neben dem Typ. Jede Oberfläche führt ihre eigene:
- Der Exportdialog der App kennt die Papiergrößen „a4“ und „a3“. Er weiß auch selbst, welche Formate nach dem Papier fragen.
- Die Verbindung zur Sitzung übersetzt diese Wörter von Hand. Ein unbekanntes Wort wird dabei still zu A4 oder zur Schnittfolge.
- Die Kommandozeile hat eigene Aufzählungen für Format und Papier und eine eigene Zuordnung von Dateiendung zu Format.

`ARCHITECTURE.md` verlangt, dass die Namen einer Auswahl einmal neben dem Typ stehen und die Oberflächen sie von dort bekommen. Dass die Listen schon auseinanderlaufen, ist sichtbar:
- Die Sitzung kann ODS, XLSX und Bildfolgen schreiben, die App bietet sie nicht an. `planung/STAND.md` führt das als offen.
- Die App kennt kein Letter-Papier, die Kommandozeile schon.
- Die Kommandozeile kann kein CSV, die App schon.

Wer die offenen Menüeinträge nachrüstet, muss heute vier Stellen in der App ändern. Diese Stellen nennt `ARCHITECTURE.md` nur zum Teil.

Vorschlag: Format, Papier und Reihenfolge bekommen Namen und eine vollständige Liste neben ihrem Typ, so wie Ebenen und Lichtarten. Die Sitzung gibt die Liste samt der Angabe „braucht Papier“ an App und Kommandozeile. Aufwand klein bis mittel. Risiko gering: `tst_exportdialog.qml` und `crates/cli/tests/export.rs` sind vorhanden.

Beleg: `crates/app/qml/ExportDialog.qml:22`, `crates/app/qml/ExportDialog.qml:23`, `crates/app/src/bridge/export.rs:49`, `crates/app/src/bridge/export.rs:82`, `crates/app/src/bridge/export.rs:90`, `crates/cli/src/main.rs:64`, `crates/cli/src/main.rs:77`, `crates/cli/src/main.rs:216`, `crates/export/src/page.rs:6`, `planung/STAND.md:16`, `ARCHITECTURE.md:63`.

### 2. Die Verbindungsdatei zwischen Fenster und Sitzung liegt knapp unter der Grenze; die Mark-Karte hat einen eigenen Aufruf je Feld
`bridge.rs` deklariert alle Aufrufe, die das Fenster an die Sitzung stellen kann. Die Datei hat 784 Zeilen, die Grenze liegt bei 800. Seit dem letzten Audit wurde sie in 6 Commits geändert. Jede neue Funktion des Fensters fügt hier Zeilen hinzu.

Ein Teil des Wachstums ist selbst gemacht. Kamera- und Lampenkarte haben je einen einzigen Aufruf „setze Feld X auf Wert Y“. Die Mark-Karte hat dagegen einen eigenen Aufruf je Feld: Haltung, Pause, Ankunft, Abgang, Fahrweise, Objektiv, Tempo. Fahrweise und Objektiv sind mit den Kamera-Marks neu dazugekommen. Jedes Feld kostet so etwa sechs Zeilen hier und neun in `bridge/marks.rs`. Ein neues Kartenfeld wird damit je nach Karte auf eine von zwei Arten gebaut. Die nächste größere Funktion bringt die Datei über die Grenze.

Vorschlag: Die Mark-Karte bekommt wie Kamera und Lampe einen einzigen Aufruf mit Feldnamen. Das spart rund 40 Zeilen und macht neue Mark-Felder zu einer Zeile in `mark.rs`. Danach ist zu prüfen, ob die Projektverwaltung (Szenen, Drehorte, Figuren, Pläne, zuletzt geöffnete Projekte) ein eigenes Objekt für die Oberfläche bekommt. Aufwand klein für den ersten Schritt, mittel für die Teilung. Risiko gering: `tst_markcard.qml` und die Tests in `mark.rs` sind vorhanden.

Beleg: `crates/app/src/bridge.rs:305`, `crates/app/src/bridge.rs:312`, `crates/app/src/bridge.rs:334`, `crates/app/src/bridge.rs:359`, `crates/app/src/bridge.rs:366`, `crates/app/src/bridge.rs:372`, `crates/app/src/bridge/marks.rs:24`, `crates/app/src/bridge/marks.rs:37`, `crates/app/src/bridge/marks.rs:55`, `crates/app/src/bridge/marks.rs:89`.

### 3. Die Lampenkarte ist eine Kopie der Kamerakarte
Mit LH-50 kam die Lampenkarte dazu. Drei Teile stehen in Kamera- und Lampenkarte fast wortgleich:
- wie die Karte die Werte des gewählten Elements holt,
- wie sie ihre Felder füllt,
- wie sie getippte Werte übernimmt, also mehrere Felder nacheinander setzt, abgelehnte markiert und die Karte erst am Ende neu füllt.

Die Übernahme sind je rund 30 Zeilen mit einer heiklen Reihenfolge, die ein langer Kommentar in beiden Dateien erklärt. Sie unterscheiden sich nur im Namen des Aufrufs. Ein Fehler in dieser Reihenfolge muss in beiden Karten gefunden und behoben werden. Jede weitere Karte, etwa für Objekte, würde die Kopie ein drittes Mal anlegen.

Vorschlag: Die Übernahme der Felder wird ein gemeinsamer Baustein der Karten. Er bekommt den Aufruf, der ein Feld setzt. Aufwand klein. Risiko gering: `tst_cameracard.qml` und `tst_lightcard.qml` prüfen das Übernehmen.

Beleg: `crates/app/qml/CameraCard.qml:19`, `crates/app/qml/CameraCard.qml:61`, `crates/app/qml/CameraCard.qml:87`, `crates/app/qml/LightCard.qml:19`, `crates/app/qml/LightCard.qml:55`, `crates/app/qml/LightCard.qml:74`.

### 4. Die Grenzwerte einer Lampe werden an zwei Stellen geprüft
Für Leistung, Farbtemperatur und Abstrahlwinkel einer Lampe gibt es einen Höchst- und einen Mindestwert. Die Grenzwerte selbst stehen einmal im Modell. Die Prüfung „liegt der Wert innerhalb?“ ist aber zweimal geschrieben: einmal in der Ablage für die mitgelieferte Lampenbibliothek, einmal in der Sitzung für die Lampenkarte. Bei Möbeln ist das schon gelöst, dort liegen die Prüfungen als Funktionen im Modell und beide Stellen rufen sie auf. Ändert sich eine Regel, zum Beispiel ob ein Winkel von 0 Grad erlaubt ist, muss das an beiden Stellen gleich geschehen. Sonst lädt die Bibliothek eine Lampe, die die Karte ablehnt, oder umgekehrt. Genau dieser Fall war Anlass für einen Korrektur-Commit in diesem PR („Check library lamps by the card's limits“).

Vorschlag: Die Prüfungen kommen als kleine Funktionen neben die Grenzwerte ins Modell, wie bei den Möbeln. Ablage und Sitzung rufen sie auf. Aufwand klein. Risiko gering: `store/src/light.rs` und `services/tests/light.rs` testen beide Seiten.

Beleg: `crates/store/src/light.rs:42`, `crates/services/src/light_edit.rs:63`, `crates/model/src/light_preset.rs:6`, `crates/model/src/library.rs:7`, Commit `bb9c708`.

### 5. Der Winkel beim Messen wird in der App berechnet, mit eigener Kopie zweier vorhandener Rechnungen
Seit LH-14 zeigt das Maßband auch den Winkel. Die App rechnet die Richtung zwischen den zwei Punkten selbst aus und bringt sie selbst in den Bereich 0 bis 360 Grad. Beides gibt es schon:
- die Richtung als Funktion des Punkts im Modell, genutzt in Stellprobe und Sitzung,
- das Einordnen in den Bereich in den Planänderungen der Sitzung, samt derselben Konstante für die volle Umdrehung.

`ARCHITECTURE.md` sagt, dass Geometrie und Berechnungen nicht in eine Oberfläche gehören. Ändert sich, wie Winkel gezählt werden, zeigt das Maßband etwas anderes als die Karten.

Vorschlag: Die Messung nutzt die vorhandene Richtungsfunktion. Das Einordnen in den Bereich steht einmal im Modell neben ihr, und Sitzung und Maßband nutzen es beide. In der App bleibt nur das Runden für die Anzeige. Aufwand klein. Risiko gering: Die Winkelanzeige ist in `app/src/tools/tests.rs` getestet.

Beleg: `crates/app/src/tools.rs:23`, `crates/app/src/tools.rs:360`, `crates/model/src/geometry.rs:29`, `crates/services/src/plan_edit.rs:270`, `crates/services/src/plan_edit.rs:555`, `ARCHITECTURE.md:59`.

### 6. Bestandsliste: was seit dem letzten Audit gewachsen ist und was knapp unter der Grenze liegt
Die Einschätzung aus dem letzten Audit gilt weiter, siehe unten. Neu ist Folgendes.

**Gewachsen, aber in Ordnung:**
- Die Feldnamen (`field.rs`, 34 auf 38 Verzweigungen) und die Liste der Aufrufe (`switched`) bleiben flache Tabellen.
- Die Prüfung der Planänderungen (`check`) ist von 20 auf 21 Verzweigungen gewachsen. Sie wählt weiter nur je Art der Änderung eine Prüfung.

**Gewachsen und an einer Naht trennbar:** Die Datei der Planänderungen `plan_edit.rs` hat 764 Zeilen und wurde seit dem Audit in 9 Commits geändert. Ihr Verteiler `apply` ist von 122 auf 125 Zeilen gewachsen. Die Datei enthält auch das Anklicken von Elementen (was unter dem Zeiger liegt, was im Rahmen liegt, was greifbar ist). Das Anklicken ist keine Änderung. Es wird nur von `session/pick.rs` benutzt und kann dorthin umziehen. Das gibt der Datei Luft vor der Grenze.

**Knapp unter der Grenze, ohne natürliche Naht:** Zwei Testdateien sind fast voll, `export/tests/export.rs` mit 796 Zeilen und `services/tests/session.rs` mit 787 Zeilen. Der nächste Test gehört in eine neue Datei nach Thema, so wie es `services/tests/` mit `rig.rs`, `light.rs` und `grid.rs` schon macht.

**Geschrumpft:** `compose` im Export ist aus der Liste verschwunden.

Aufwand klein. Risiko gering: `services/tests/plan_edit.rs` und die Tests des Auswählens in der Sitzung decken das Anklicken ab.

Beleg: `crates/services/src/plan_edit.rs:267`, `crates/services/src/plan_edit.rs:274`, `crates/services/src/plan_edit.rs:285`, `crates/services/src/plan_edit.rs:316`, `crates/services/src/plan_edit.rs:427`, `crates/services/src/session/pick.rs:124`, `crates/services/src/session/pick.rs:132`, `crates/services/src/field.rs:80`, `structure.baseline`.

## Seit dem letzten Audit
Der Eigentümer hat über die Funde des dritten Audits noch nicht entschieden. Die offenen Funde stehen deshalb hier und werden nicht neu gemeldet.

- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): offen, nicht entschieden.** Die Zuordnung von Fehler zu Text steht weiter nur in der App. Terminalprogramm und Kommandozeile zeigen den inneren englischen Text. Beleg: `crates/app/src/notice.rs:8`, `crates/tui/src/app.rs:354`, `crates/cli/src/main.rs:167`, `crates/cli/src/main.rs:174`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen, nicht entschieden.** `Workspace.qml` ist bei den doppelten Teilen unverändert. Beleg: `crates/app/qml/Workspace.qml:146`, `crates/app/qml/Workspace.qml:242`, `crates/app/qml/Workspace.qml:265`.
- **Fund 3 (Datei für Änderungen am Projekt mit drei Aufgaben): teilweise behoben.** Die Planvarianten liegen jetzt in einer eigenen Datei. Anlass war die Größengrenze, nicht eine Entscheidung zum Fund. `project_edit.rs` hat noch 626 Zeilen. Der Import in die Objektbibliothek steht weiter darin. Das ist nicht mehr dringend, der Vorschlag gilt für den Rest. Beleg: `crates/services/src/project_edit/plans.rs`, `crates/services/src/project_edit.rs:361`, Commit `281a9c9`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen, nicht entschieden.** Der neue Wert für die Rasterweite wurde richtig in beide Listen eingetragen. Die Listen werden weiter nur von Hand gleich gehalten. Beleg: `crates/app/src/bridge/view.rs:110`, `crates/app/src/bridge/view.rs:384`.
- **Fund 5 (Bestandsliste): offen, nicht entschieden.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert und trennbar wie beschrieben. Was damals als „in Ordnung“ galt, gilt weiter. Ergänzungen stehen im neuen Fund 6.
- Abgelehnt hat der Eigentümer keinen Fund.

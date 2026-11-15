# Code-Audit Klappe, 15.11.2026, Stand e4100e9

## Urteil
Das Repo bleibt wartbar. Die Schichten halten: Jedes Paket hängt nur von den Paketen ab, die `ARCHITECTURE.md` erlaubt, und die neue Kopie von Kante stimmt mit dem Original überein. Gegenüber dem vierten Audit ist der Zustand etwas schlechter. Kein Fund wurde auf Entscheidung des Eigentümers behoben. Zwei Dateien haben Luft bekommen, weil Teile wegen der Größengrenze in eigene Dateien umgezogen sind. Gleichzeitig sind fünf Einträge der Bestandsliste gewachsen und einer ist neu dazugekommen. Die Vorlagen (LH-07) haben neue Doppelungen gebracht: Die Grenzwerte für Kameras werden in Ablage und Sitzung je eigens geprüft, und das Verschieben von Marks steht an drei Stellen. Eine Regel für Figurennamen steht dreimal im Code. Dass solche Kopien auseinanderlaufen, hat in diesem Zeitraum schon zwei Korrektur-Commits gekostet.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 260 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung) | 297 |
| Einträge in der Bestandsliste | 31 | 31 (einer weg: `tst_planarea.qml`; einer neu: `look` in Stellprobe) |
| größte Datei (Zeilen) | 1534 (`crates/stellprobe/src/display.rs`) | 1582 (dieselbe Datei, davon 880 Zeilen Tests) |
| längste Funktion (Zeilen) | 130 (`plan` in einem Test der Ablage); ohne Tests 125 (`apply` in `plan_edit.rs`) | 131 (`plan` im Test und `apply` in `plan_edit.rs` gleichauf). `switched` ist von 496 auf 544 Zeilen gewachsen. Es ist weiter keine Funktion, sondern die Liste der Aufrufe der Verbindung zur Oberfläche. |

## Funde

### 1. Ob zwei Figurennamen gleich sind, steht an drei Stellen
Zwei Figuren dürfen keinen Namen haben, der im Drehbuch dieselbe Sprecherzeile ergibt. „Strauss“ und „Strauß“ sind zum Beispiel beide „STRAUSS“. Diese Regel steht dreimal im Code:
- beim Erkennen der Sprecherzeilen im Drehbuch,
- bei der Prüfung, ob ein neuer Figurenname noch frei ist,
- beim Einsetzen einer Vorlage, das für einen Platzhalter einen freien Namen sucht („A“, „A 2“ …).

Die Stellen sind schon auseinandergelaufen. Die Namensprüfung hat in Kleinbuchstaben verglichen, das Drehbuch in Großbuchstaben. Dadurch flossen die Zeilen zweier Figuren im Drehbuch zusammen. Behoben wurde das im Commit `45db506`, eine Stelle von Hand an die andere angeglichen. Die dritte Stelle ist mit den Vorlagen neu dazugekommen. Sie muss zur Namensprüfung passen, sonst lehnt das Einsetzen einer Vorlage die eigene Figur ab. Schon heute unterscheiden sich die Stellen in einem Detail: Nur das Drehbuch entfernt vorher Leerzeichen am Rand.

Vorschlag: eine Funktion „gleicher Sprechername“ neben dem Drehbuch-Code, die alle drei Stellen aufrufen. Aufwand klein. Risiko gering: `services/tests/rename.rs` (`names_that_share_a_cue_are_taken`) und `services/tests/templates.rs` prüfen beide Seiten.

Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:310`, `crates/services/src/plan_edit/template.rs:142`, `crates/services/tests/rename.rs:186`, Commit `45db506`.

### 2. Einstellungsgröße und Perspektive stehen als eigene Liste in der App
Jeder Shot hat drei Auswahlen: Einstellungsgröße (Totale bis Detail), Perspektive (Normalsicht, Aufsicht …) und Kamerabewegung. Für die Bewegung holt die App die Liste aus der Sitzung. Die beiden anderen Listen hat sie selbst abgeschrieben. Das widerspricht `ARCHITECTURE.md`: Die Namen einer Auswahl stehen einmal neben dem Typ, und die Oberflächen bekommen sie von dort. Mit der neuen Shot-Tabelle (D-05) nutzt jetzt auch eine zweite Ansicht die abgeschriebene Liste.

Kommt eine Einstellungsgröße dazu oder wird eine umbenannt, zeigt die App sie nicht an, oder sie schickt einen Namen, den die Sitzung ablehnt. Kein Test bemerkt das, weil die Listen nirgends verglichen werden. Die Tabelle in `ARCHITECTURE.md` sagt für eine neue Auswahl „nichts in einer Oberfläche“. Hier stimmt das nicht.

Vorschlag: Größe und Perspektive kommen wie die Bewegung in die Liste der Auswahlen, die die Verbindung an die Oberfläche gibt. Die App liest sie von dort. Aufwand klein. Risiko gering: `tst_shotlist.qml` und der Test der Auswahlliste in `bridge/view.rs` sind vorhanden.

Beleg: `crates/app/qml/ShotPanel.qml:87`, `crates/app/qml/ShotPanel.qml:88`, `crates/app/qml/ShotPanel.qml:89`, `crates/app/qml/ShotTable.qml:117`, `crates/app/qml/ShotTable.qml:118`, `crates/app/src/bridge/view.rs:285`, `crates/app/src/bridge/view.rs:306`, `crates/model/src/shot.rs:64`, `crates/model/src/shot.rs:110`, `ARCHITECTURE.md:63`, `ARCHITECTURE.md:77`.

### 3. Die Grenzwerte für Kamera und Marks werden in Ablage und Sitzung je eigens geprüft, beim Objektiv mit zwei verschiedenen Höchstwerten
Mit den Vorlagen kamen die Grenzwerte für Objektiv, Sensor, Bildformat, Kamerahöhe, Neigung und Planfläche als Zahlen ins Modell. Das ist gut. Die Prüfung „liegt der Wert innerhalb?“ steht aber zweimal:
- einmal in der Sitzung für die Karten,
- einmal in der Ablage für die Vorlagen.

Das gilt auch für die Pause an einem Mark. Bei Möbeln liegen solche Prüfungen schon als Funktionen im Modell, die beide Seiten aufrufen. Wie teuer die Kopie ist, zeigt der Korrektur-Commit „Hold templates to the editor's bounds“ (`0c782f3`) in diesem Zeitraum. Dort wurde die Ablage nachträglich an die Karten angeglichen. Der offene Fund 4 des letzten Audits beschreibt denselben Mangel für Lampen.

Dazu kommt eine dritte Grenze für dieselbe Sache. Das Objektiv eines Shots darf höchstens 2000 mm haben, mit einer eigenen Konstante gleichen Namens. Die Kamera und ihre Marks erlauben bis 5000 mm. Eine Kamera mit 3000 mm ist erlaubt. Tippt man denselben Wert in die Objektivspalte eines Shots, wird er abgelehnt.

Vorschlag: Die Prüfungen kommen als kleine Funktionen neben die Grenzwerte ins Modell, wie bei Möbeln. Sitzung und Ablage rufen sie auf. Der Eigentümer entscheidet, ob ein Shot dieselbe Objektivgrenze hat wie die Kamera. Am besten zusammen mit dem offenen Fund 4 zu den Lampen beheben. Aufwand klein bis mittel. Risiko gering: `store/tests/templates.rs`, `services/tests/templates.rs`, `services/tests/camera.rs` und die Tests in `edit.rs` decken beide Seiten ab.

Beleg: `crates/model/src/limits.rs:18`, `crates/store/src/template.rs:67`, `crates/store/src/template.rs:87`, `crates/store/src/template.rs:92`, `crates/store/src/template.rs:98`, `crates/services/src/camera_edit.rs:18`, `crates/services/src/camera_edit.rs:34`, `crates/services/src/plan_edit/check.rs:96`, `crates/services/src/plan_edit/check.rs:108`, `crates/services/src/edit.rs:62`, `crates/services/src/edit.rs:63`, `crates/model/src/library.rs:7`, Commit `0c782f3`.

### 4. Marks verschieben und die Mitte einer Gruppe finden ist mehrfach geschrieben
Drei Vorgänge versetzen eine Gruppe von Figuren und Kameras um ein Stück:
- Kopieren mehrerer Elemente,
- Einsetzen einer Vorlage,
- Ablegen eines Plans als Vorlage.

Jeder hat seine eigene Funktion, die einen Punkt verschiebt, und eigene Funktionen, die einen Mark samt Umweg verschieben. Die Funktionen gibt es jeweils für Figuren-Marks und für Kamera-Marks. Das sind sechs fast gleiche Funktionen in drei Dateien. Auch die Mitte des Rahmens um eine Gruppe von Punkten ist zweimal geschrieben, in zwei Dateien. Dazu kommt eine Funktion „Punkt drehen“, die nur die gleichnamige Funktion des Modells aufruft.

Bekommt ein Mark einen weiteren Ort, etwa einen zweiten Umweg, muss das an allen drei Stellen ergänzt werden. Vergisst man eine, bleibt beim Kopieren oder beim Einsetzen einer Vorlage ein Teil des Wegs an der alten Stelle liegen.

Vorschlag: Der Punkt bekommt im Modell ein „verschoben um“, Figuren- und Kamera-Mark je ein „verschoben um“, und die Mitte des Rahmens steht einmal neben der Geometrie des Modells. Die drei Stellen rufen diese Funktionen auf, die Drehfunktion ohne eigene Arbeit entfällt. Aufwand klein. Risiko gering: `services/tests/templates.rs` und `services/tests/multisel.rs` decken Einsetzen, Ablegen und Kopieren ab.

Beleg: `crates/services/src/plan_edit/group.rs:184`, `crates/services/src/plan_edit/group.rs:226`, `crates/services/src/plan_edit/group.rs:293`, `crates/services/src/plan_edit/template.rs:115`, `crates/services/src/plan_edit/template.rs:119`, `crates/services/src/plan_edit/template.rs:127`, `crates/services/src/project_edit/templates.rs:141`, `crates/services/src/project_edit/templates.rs:155`, `crates/services/src/project_edit/templates.rs:160`, `crates/services/src/project_edit/templates.rs:168`, `crates/services/src/plan_edit/moving.rs:448`, `crates/model/src/geometry.rs:29`.

### 5. Das Einsetzen einer Vorlage liegt bei den Planänderungen und ruft die Projektänderungen auf, die es selbst auslösen
Eine Vorlage einsetzen ist in der Sitzung eine Änderung am Projekt, weil dabei neue Figuren entstehen können. Sie wird auch von den Projektänderungen ausgelöst. Der Code dafür liegt aber bei den Planänderungen. Für neue Figuren ruft er wieder zurück in die Projektänderungen. Damit verweisen die beiden Bereiche gegenseitig aufeinander, also im Kreis. Wer am Einsetzen etwas ändert, muss beide Bereiche kennen und vier Dateien zu Vorlagen in der Sitzung auseinanderhalten (`templates.rs`, `session/templates.rs`, `project_edit/templates.rs`, `plan_edit/template.rs`).

Vorschlag: Das Einsetzen zieht in die Datei der Vorlagen bei den Projektänderungen. Dort wird es schon ausgelöst und dort entstehen die neuen Figuren. Von den Planänderungen braucht es dann nur noch die Suche nach einem freien Kamerabuchstaben, in eine Richtung. Aufwand klein. Risiko gering: `services/tests/templates.rs` prüft Einsetzen und Ablegen.

Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/plan_edit/template.rs:15`, `crates/services/src/project_edit/templates.rs:11`, `crates/services/src/project_edit/templates.rs:20`, `crates/services/src/project_edit/templates.rs:184`, `crates/services/src/project_edit.rs:91`, `crates/services/src/project_edit.rs:175`.

### 6. Bestandsliste: was gewachsen ist und was knapp unter der Grenze liegt
Die Einschätzung der letzten beiden Audits gilt weiter, siehe unten. Neu ist Folgendes.

**Gewachsen und an einer Naht trennbar:**
- Der Verteiler der Planänderungen `apply` hat jetzt 131 statt 125 Zeilen. Er wächst mit jeder neuen Art der Planänderung, ebenso seine Prüfung `check`.
- Das Ziel einer Verschiebung `target` hat jetzt 126 statt 116 Zeilen. Es wählt je Art des gegriffenen Elements aus, Figuren- und Kamera-Marks in fast gleichen Zweigen. Das teilt sich nach Element: Marks, Wände, Zeichnungen.
- Fund 4 oben spart in `group.rs` einige Zeilen von `copy`.

**Gewachsen, aber in Ordnung:**
- `switched` (544 Zeilen) bleibt eine flache Liste.
- `error_text` (24 Verzweigungen) bleibt eine flache Zuordnung. Ihr Ort ist Fund 1 des dritten Audits.
- Neu in der Liste ist `look` in Stellprobe (16 Verzweigungen). Es ist eine flache Tabelle „Rolle → Aussehen“, wie `ARCHITECTURE.md` sie will.
- `display.rs` ist auf 1582 Zeilen gewachsen, davon 880 Zeilen Tests.

**Knapp unter der Grenze von 800 Zeilen:**
- `services/tests/library.rs` hat genau 800 Zeilen. Der nächste Test gehört in eine neue Datei nach Thema, wie es die neuen `templates.rs`, `retime.rs` und `drawing_grips.rs` schon machen.
- `export/tests/export.rs` hat 798 Zeilen, `services/tests/session.rs` 788. Für beide gilt dasselbe.
- `model/src/plan.rs` hat 787 Zeilen, davon 214 Tests. Die Datei hat eine natürliche Naht: Zeichnungen samt Strichart und Füllung sowie Lampen samt Lichtart können je in eine eigene Datei, wie es Shots und Vorlagen schon sind.
- `bridge/view.rs` hat 748 Zeilen, `bridge.rs` 746.

Aufwand klein je Datei. Risiko gering: `services/tests/plan_edit.rs`, `drawing_grips.rs`, `curves.rs` und die Tests im Modell decken die Stellen ab.

Beleg: `crates/services/src/plan_edit.rs` (`apply`), `crates/services/src/plan_edit/moving.rs:10`, `crates/stellprobe/src/look.rs:55`, `crates/services/tests/library.rs`, `crates/export/tests/export.rs`, `crates/services/tests/session.rs`, `crates/model/src/plan.rs:95`, `crates/model/src/plan.rs:490`, `structure.baseline`.

## Seit dem letzten Audit
Der Eigentümer hat über die Funde des dritten und vierten Audits noch nicht entschieden. Die offenen Funde stehen deshalb hier und werden nicht neu gemeldet. Abgelehnt hat er keinen Fund.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): offen.** Die Zuordnung von Fehler zu Text steht weiter nur in der App und ist auf 24 Verzweigungen gewachsen. Die Kommandozeile gibt weiter eigene englische Sätze aus. Beleg: `crates/app/src/notice.rs:8`, `crates/cli/src/main.rs:166`, `crates/cli/src/main.rs:173`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen.** `Workspace.qml` ist bei den doppelten Teilen unverändert. Beleg: `crates/app/qml/Workspace.qml:114`, `crates/app/qml/Workspace.qml:125`.
- **Fund 3 (Datei für Änderungen am Projekt mit mehreren Aufgaben): teilweise behoben, wieder gewachsen.** Der Import in die Objektbibliothek steht weiter in `project_edit.rs`. Die Datei ist von 626 auf 687 Zeilen gewachsen, weil die eigenen Objekte (LH-12) dazukamen. Beleg: `crates/services/src/project_edit.rs:393`, `crates/services/src/project_edit.rs:429`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen.** Beleg: `crates/app/src/bridge/view.rs`.
- **Fund 5 (Bestandsliste): offen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): offen.** Die Listen in Exportdialog, Verbindung und Kommandozeile sind unverändert. Beleg: `crates/app/qml/ExportDialog.qml:22`, `crates/app/src/bridge/export.rs:82`, `crates/cli/src/main.rs:64`.
- **Fund 2 (Verbindungsdatei knapp unter der Grenze, ein Aufruf je Mark-Feld): teilweise entlastet, Kern offen.** Der Start des Fensters ist nach `bridge/start.rs` umgezogen. `bridge.rs` hat jetzt 746 statt 784 Zeilen. Anlass war die Größengrenze, nicht eine Entscheidung zum Fund. Die sieben Einzelaufrufe der Mark-Karte bestehen weiter. Beleg: `crates/app/src/bridge.rs:349`, `crates/app/src/bridge.rs:387`, Commit `7b37b55`.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen.** Die Übernahme der Felder steht weiter in beiden Karten. Beleg: `crates/app/qml/CameraCard.qml:92`, `crates/app/qml/LightCard.qml:74`.
- **Fund 4 (Grenzwerte einer Lampe an zwei Stellen geprüft): offen.** Derselbe Mangel ist mit den Vorlagen für Kameras und Marks dazugekommen, siehe neuer Fund 3. Beleg: `crates/store/src/light.rs:42`, `crates/services/src/light_edit.rs:53`.
- **Fund 5 (Winkel beim Messen in der App berechnet): offen.** Beleg: `crates/app/src/tools.rs:23`, `crates/app/src/tools.rs:360`, `crates/services/src/plan_edit.rs:281`.
- **Fund 6 (Bestandsliste): teilweise behoben.** Das Anklicken von Elementen ist aus `plan_edit.rs` in eine eigene Datei `plan_edit/picking.rs` gezogen. Die Datei hat jetzt 731 statt 764 Zeilen. Der Rat, neue Tests in neue Dateien nach Thema zu legen, wurde befolgt. Der Verteiler `apply` ist trotzdem weiter gewachsen, siehe neuer Fund 6.

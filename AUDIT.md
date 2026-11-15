# Code-Audit Klappe, 15.11.2026, Stand d295047

## Urteil
Das Repo bleibt wartbar, und es steht etwas besser da als beim fünften Audit. Vier Funde wurden behoben: Die Grenzwerte für Kameras stehen jetzt einmal im Modell. Die App holt Einstellungsgröße und Perspektive aus der Sitzung. Fehlertexte kommen für Fenster und Kommandozeile aus demselben Katalog. Die Objektbibliothek hat einen eigenen Teil, und die Datei für Projektänderungen ist von 687 auf 575 Zeilen geschrumpft. Die Schichten halten: Jedes Paket hängt nur von den Paketen ab, die `ARCHITECTURE.md` erlaubt. Die neuen Funktionen nutzen zum großen Teil vorhandene Bausteine: Die Figurengröße wird in Ablage und Sitzung mit derselben Prüfung aus dem Modell geprüft, und Grundriss und Kamerablick schneiden Türen mit derselben Funktion aus der Wand. Auf der Gegenseite stehen drei neue kleine Doppelungen, siehe Funde 1 bis 3. Außerdem ist die Bestandsliste wieder gewachsen: Sieben Einträge sind größer geworden, einer ist neu.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 297 (ohne die Kopie von Kante, so gezählt wie die Strukturprüfung) | 316 |
| Einträge in der Bestandsliste | 31 | 32 (neu: `load` in der Ablage; `error_text` ist mit gleicher Größe aus der App in die Sitzung umgezogen) |
| größte Datei (Zeilen) | 1582 (`crates/stellprobe/src/display.rs`) | 1559 (dieselbe Datei) |
| längste Funktion (Zeilen) | 131 (`plan` im Test der Ablage und `apply` in `plan_edit.rs` gleichauf) | 134 (`apply` in `plan_edit.rs`). `switched` ist von 544 auf 558 Zeilen gewachsen. Es ist weiter keine Funktion, sondern die Liste der Aufrufe der Verbindung zur Oberfläche. |

## Funde

### 1. Die Großschreibung beim Tippen (DB-01) kopiert Regeln des Drehbuchlesers in das Terminalprogramm
Beim Tippen werden Szenenköpfe und mit `@` erzwungene Figurennamen großgeschrieben. Dafür muss das Programm wissen, wo im Szenenkopf die Szenennummer beginnt (`#12a#`). Es muss auch wissen, wo eine Erweiterung wie „(cont'd)“ beginnt und dass „ß“ klein bleibt. Diese Regeln stehen schon im Leser des Drehbuchs, also dem Teil, der aus dem Text die Elemente macht. Das Terminalprogramm hat sie mit eigenen Konstanten nachgeschrieben. Eine der Regeln lautet zum Beispiel: Ein `#` mit einem Leerzeichen dahinter beginnt keine Nummer.

Was das kostet, war schon beim Entstehen zu sehen. Die Kopie musste zweimal an den Leser angeglichen werden. Einmal wurde „#3a#“ zu „#3A#“, und damit änderte sich die Nummer, auf die Shots verweisen. Einmal blieb bei „int. büro #2 flur“ das Wort „flur“ klein. Ändert sich eine Fountain-Regel im Leser, muss die Kopie von Hand mitgezogen werden. Das Fenster bekommt laut `ARCHITECTURE.md` noch einen eigenen Drehbuch-Editor (DB-23). Der braucht dieselbe Großschreibung, kommt aber an den Code im Terminalprogramm nicht heran. Es entstünde eine dritte Kopie.

Vorschlag: Die Großschreibung einer Zeile kommt neben den Leser in das Drehbuch-Paket, nutzt dessen Konstanten und dessen Regel für Nummern und wird über die Sitzung angeboten. Das Terminalprogramm ruft sie nur auf, später auch das Fenster. Aufwand klein. Risiko gering: Die Tests in `tui/src/fountain.rs` und `tui/src/tests.rs` prüfen jeden bisher gefundenen Fall.

Beleg: `crates/tui/src/fountain.rs:121`, `crates/tui/src/fountain.rs:122`, `crates/tui/src/fountain.rs:124`, `crates/tui/src/fountain.rs:128`, `crates/tui/src/fountain.rs:151`, `crates/tui/src/fountain.rs:162`, `crates/tui/src/fountain.rs:167`, `crates/script/src/parse.rs:9`, `crates/script/src/parse.rs:15`, `crates/script/src/parse.rs:17`, `crates/script/src/parse.rs:374`, `crates/script/src/parse.rs:393`, `crates/script/src/parse.rs:407`, `ARCHITECTURE.md` (Abschnitt „Known deviations“), Commits `80bf207`, `d037105`.

### 2. Die Breite von Türen und Fenstern prüfen Ablage und Sitzung je für sich, mit verschiedener Regel
Mit Fund 3 des fünften Audits wurde für Kameras und Marks festgelegt: Die Prüfung „liegt der Wert innerhalb?“ steht einmal im Modell, und Ablage und Sitzung rufen sie auf. Bei Türen und Fenstern (LH-10) ist das nicht geschehen. Die Ablage prüft beim Öffnen eines Projekts selbst nur, ob die Breite eine Zahl über null ist. Die Sitzung prüft beim Eintippen dagegen mit der Regel für die Seitenlänge eines Möbelstücks, also höchstens 1000 m. Eine Tür von 2000 m Breite in der Datei wird deshalb ohne Fehler geöffnet und gezeichnet. Tippt man denselben Wert in die Karte der Tür, wird er abgelehnt. Dazu trägt die Prüfung in der Sitzung den Namen „Größe eines Möbelstücks“. Wer eine Grenze für Türen ändern will, sucht dort nicht danach.

Bei der Figurengröße (LH-25) aus demselben Zeitraum ist es richtig gelöst: Ablage und Sitzung rufen beide `is_figure_size_m` aus dem Modell auf.

Vorschlag: Eine Prüfung „ist eine Türbreite“ kommt neben die Türen ins Modell, und Ablage und Sitzung rufen sie auf. Das lässt sich gut zusammen mit dem offenen Fund 4 des vierten Audits beheben, der denselben Mangel für Lampen beschreibt. Aufwand klein. Risiko gering: `store/tests/openings.rs` und `services/tests/openings.rs` decken beide Seiten ab.

Beleg: `crates/store/src/lib.rs:356`, `crates/store/src/lib.rs:361`, `crates/services/src/plan_edit/check.rs:156`, `crates/services/src/plan_edit/check.rs:157`, `crates/model/src/library.rs:7`, `crates/model/src/project.rs:271`, zum Vergleich `crates/store/src/lib.rs:341` und `crates/services/src/project_edit.rs:471`.

### 3. Die Spalte „Achsensprung gegen“ wird in Sitzung und Ausgabe je für sich geschrieben
Springt ein Shot über die 180°-Achse, nennen Shot-Tabelle und ausgegebene Shotliste die Nummern der Shots auf der anderen Seite, etwa „12.1, 12.2“. Diesen Text bauen zwei Funktionen in zwei Paketen, jede mit eigener Konstante für das Trennzeichen. In einem Detail unterscheiden sie sich schon: Findet die Ausgabe einen Shot nicht, schreibt sie dessen inneren Namen hin, die Sitzung lässt ihn weg. Soll die Spalte anders aussehen, etwa mit Semikolon oder „gegen 12.1“, muss man beide Stellen ändern. Sonst zeigen Fenster und Ausdruck Verschiedenes.

Vorschlag: Der Text kommt einmal neben die Berechnung der Achsensprünge in Stellprobe (zum Beispiel als Nummernliste am Achsensprung), und beide Stellen verwenden ihn. Aufwand klein. Risiko gering: `services/tests/axis.rs` und `export/tests/shots.rs` prüfen den Text auf beiden Seiten.

Beleg: `crates/services/src/session/shots.rs:11`, `crates/services/src/session/shots.rs:14`, `crates/export/src/shots.rs:21`, `crates/export/src/shots.rs:77`, `crates/stellprobe/src/axis.rs:71`, `crates/stellprobe/src/axis.rs:83`.

### 4. Bestandsliste: Die Verteiler der Planänderungen wachsen mit jeder neuen Art von Element
Die Einschätzung der letzten Audits gilt weiter, siehe unten bei Fund 6 des fünften Audits. Neu ist Folgendes.

**Gewachsen, an einer Naht trennbar:** Mit Türen und Fenstern hat jede Funktion, die nach der Art des Elements verzweigt, einen Zweig mehr bekommen:
- `apply`: 134 statt 131 Zeilen, 42 statt 41 Verzweigungen,
- `check`: 22 statt 21,
- das Ziel einer Verschiebung `target`: 130 statt 126 Zeilen,
- `remove`: 21 statt 19,
- `copy`: 86 statt 85 Zeilen.

Die nächste Art von Element verlängert wieder alle fünf. Für `target` bleibt der Vorschlag des letzten Audits: nach Element trennen. Die Türen zeigen, dass das geht: Ihr Hinzufügen und ihre Breite stehen schon in einer eigenen Datei `plan_edit/openings.rs`, und `apply` gibt nur weiter.

**Neu in der Liste:** Das Laden eines Projekts in der Ablage (`load`, 16 Verzweigungen) kam mit den Prüfungen für Figurengröße und Türen dazu. Das ist noch eine flache Folge von Schritten. Kommt eine weitere Prüfung beim Laden dazu, gehören die Prüfungen in einen eigenen Schritt.

**Knapp unter der Grenze von 800 Zeilen:** Das sind jetzt neun Dateien. Gewachsen sind die Tests der Projektverwaltung `services/tests/manage.rs` (796 statt 702), `bridge/view.rs` (777 statt 748), `bridge.rs` (764 statt 746), `app/src/plan.rs` (762 statt 748) und `plan_edit.rs` (755 statt 731). Unverändert knapp sind `services/tests/library.rs` (800), `export/tests/export.rs` (795), `services/tests/session.rs` (788) und `model/src/plan.rs` (787). Der Rat, neue Tests in neue Dateien nach Thema zu legen, wurde meist befolgt: `openings.rs`, `axis.rs`, `camera_view.rs` und `figure_size.rs` sind neu. Nur die Tests der Projektverwaltung sind stattdessen um fast 100 Zeilen gewachsen.

Aufwand klein je Stelle. Risiko gering: `services/tests/plan_edit.rs`, `openings.rs`, `drawing_grips.rs` und `multisel.rs` decken die Verteiler ab.

Beleg: `crates/services/src/plan_edit.rs:409`, `crates/services/src/plan_edit.rs:487`, `crates/services/src/plan_edit/check.rs:37`, `crates/services/src/plan_edit/moving.rs:11`, `crates/services/src/plan_edit/moving.rs:110`, `crates/services/src/plan_edit/remove.rs:48`, `crates/services/src/plan_edit/group.rs:314`, `crates/store/src/lib.rs:96`, `crates/services/tests/manage.rs`, `structure.baseline`.

## Seit dem letzten Audit
Der Eigentümer hat am 15.11.2026 entschieden, dass alle Funde behoben werden. Abgelehnt hat er keinen.

Fünftes Audit:
- **Fund 1 (Gleichheit von Figurennamen an drei Stellen): offen, noch nicht begonnen.** Die drei Stellen sind unverändert. Weiter entfernt nur das Drehbuch vorher Leerzeichen am Rand. Beleg: `crates/services/src/script_refs.rs:116`, `crates/services/src/project_edit.rs:310`, `crates/services/src/plan_edit/template.rs:142`.
- **Fund 2 (Einstellungsgröße und Perspektive als eigene Liste in der App): behoben (#114).** Shot-Karte und Shot-Tabelle lesen beide Listen aus der Auswahlliste der Verbindung. Ein Test vergleicht ihre Länge mit dem Modell. Beleg: `crates/app/qml/ShotPanel.qml:99`, `crates/app/qml/ShotPanel.qml:100`, `crates/app/src/bridge/view.rs:301`, `crates/app/src/bridge/view.rs:563`.
- **Fund 3 (Grenzwerte für Kamera und Marks in Ablage und Sitzung je für sich geprüft): behoben (#113).** Die Prüfungen stehen als Funktionen im Modell. Das Objektiv eines Shots behält bewusst seine engere Grenze von 2000 mm. Sie hat jetzt einen eigenen Namen und eine Begründung, die beiden Grenzen werden also nicht mehr verwechselt. Beleg: `crates/model/src/limits.rs:31`, `crates/services/src/edit.rs:63`. Derselbe Mangel besteht noch bei Lampen (Fund 4 des vierten Audits) und ist bei Türen neu dazugekommen (neuer Fund 2).
- **Fund 4 (Marks verschieben und Mitte einer Gruppe mehrfach geschrieben): offen, noch nicht begonnen.** Die Stellen sind unverändert, in `group.rs` nur verrutscht. Beleg: `crates/services/src/plan_edit/group.rs:186`, `crates/services/src/plan_edit/group.rs:230`, `crates/services/src/plan_edit/group.rs:302`, `crates/services/src/plan_edit/template.rs:115`, `crates/services/src/plan_edit/template.rs:119`, `crates/services/src/plan_edit/template.rs:127`, `crates/services/src/project_edit/templates.rs:141`, `crates/services/src/project_edit/templates.rs:155`, `crates/services/src/project_edit/templates.rs:160`, `crates/services/src/project_edit/templates.rs:168`, `crates/services/src/plan_edit/moving.rs:464`.
- **Fund 5 (Einsetzen einer Vorlage ruft im Kreis): in Arbeit, PR #111 offen, nicht gemergt.** Auf dem Hauptstand verweisen beide Bereiche weiter gegenseitig aufeinander. Beleg: `crates/services/src/plan_edit/template.rs:8`, `crates/services/src/project_edit/templates.rs:11`, `crates/services/src/project_edit/templates.rs:184`.
- **Fund 6 (Bestandsliste): offen, noch nicht begonnen.** Die genannten Verteiler sind weiter gewachsen, siehe neuer Fund 4.

Drittes Audit:
- **Fund 1 (Fehlermeldungen nur im Fenster übersetzt): behoben (#112).** Die Zuordnung von Fehler zu Text steht jetzt in der Sitzung, und Fenster und Kommandozeile nutzen sie beide. Ein kleiner Rest bleibt: Für „keine solche Szene“ und „Szene ohne Plan“ schreibt die Kommandozeile weiter eigene englische Sätze, obwohl der Katalog beide Meldungen hat. Beleg: `crates/services/src/error_text.rs:9`, `crates/services/src/error_text.rs:10`, `crates/cli/src/main.rs:173`, `crates/cli/src/main.rs:184`.
- **Fund 2 (Drehbuch- und Shot-Spalte je zweimal gebaut): offen, noch nicht begonnen.** Beleg: `crates/app/qml/Workspace.qml:114`, `crates/app/qml/Workspace.qml:125`.
- **Fund 3 (Datei für Änderungen am Projekt mit mehreren Aufgaben): behoben (#117).** Import, Ablegen und eigene Objekte der Bibliothek stehen in einer eigenen Datei. `project_edit.rs` hat 575 statt 687 Zeilen. Beleg: `crates/services/src/project_edit/library.rs`.
- **Fund 4 (angezeigte Werte in zwei Listen ohne gegenseitige Prüfung): offen, noch nicht begonnen.** Die Datei ist von 748 auf 777 Zeilen gewachsen. Beleg: `crates/app/src/bridge/view.rs`.
- **Fund 5 (Bestandsliste): offen, noch nicht begonnen.** `item_prims` (106 Zeilen) und `scan` (98 Zeilen) sind unverändert.

Viertes Audit:
- **Fund 1 (Formate, Papier und Reihenfolge der Ausgabe je Oberfläche): offen, noch nicht begonnen.** Beleg: `crates/app/qml/ExportDialog.qml:22`, `crates/app/src/bridge/export.rs:82`, `crates/cli/src/main.rs:64`.
- **Fund 2 (Verbindungsdatei knapp unter der Grenze, ein Aufruf je Mark-Feld): offen, noch nicht begonnen.** `bridge.rs` ist von 746 auf 764 Zeilen gewachsen. Die sieben Einzelaufrufe der Mark-Karte bestehen weiter. Beleg: `crates/app/src/bridge.rs:364`, `crates/app/src/bridge.rs:402`.
- **Fund 3 (Lampenkarte als Kopie der Kamerakarte): offen, noch nicht begonnen.** Beleg: `crates/app/qml/CameraCard.qml:95`, `crates/app/qml/LightCard.qml:74`.
- **Fund 4 (Grenzwerte einer Lampe an zwei Stellen geprüft): offen, noch nicht begonnen.** Bei Türen ist derselbe Mangel neu dazugekommen, siehe neuer Fund 2. Beleg: `crates/store/src/light.rs:42`, `crates/services/src/light_edit.rs:53`.
- **Fund 5 (Winkel beim Messen in der App berechnet): offen, noch nicht begonnen.** Das Modell hat eine Richtungsfunktion. Die App rechnet trotzdem selbst. Beleg: `crates/app/src/tools.rs:372`, `crates/model/src/geometry.rs:44`.
- **Fund 6 (Bestandsliste): offen, noch nicht begonnen.** `plan_edit.rs` ist von 731 auf 755 Zeilen gewachsen, siehe neuer Fund 4.

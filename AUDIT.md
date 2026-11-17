# Code-Audit Klappe, 17.11.2026, Stand 10fdcfd2

## Urteil
Das Repo ist wartbar und besser als beim 14. Audit. Von den acht Funden des 14. Audits sind sechs ganz behoben, zwei zum größten Teil. Die Bestandsliste ist von 30 auf 26 Einträge geschrumpft, und die längste Funktion im Programmcode ist von 128 auf 120 Zeilen gesunken.

Neu ist vor allem der Abgleich des Registers mit dem Buch (#215, #220, #224). Sein Kern in der Sitzung ist sauber an einer Stelle gebaut. Am Rand ist aber wieder einiges doppelt entstanden. Die offenen Fragen zum Abgleich liegen als zweite Liste neben dem Register und werden von Hand nachgeführt. Ich habe gezeigt, dass sie dabei veralten (Fund 1). Der Dialog für diese Fragen ist im Terminal und im Fenster je für sich gebaut, mit doppelten Texten (Fund 4). Dazu ist `ARCHITECTURE.md` an zwei Stellen nicht mehr richtig (Funde 2 und 3).

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 500 | 558 (gezählt wie die Strukturprüfung, ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | 30 | 26. Weg sind `write` (PDF), `name` in `plan/key.rs`, `remove` und `target`. Kleiner geworden sind `apply`, `check`, `own_text`, `save` und die Tests der Ablage. Gewachsen ist ein Eintrag: `name` in `field.rs`, von 44 auf 45 Zweige (#215). |
| größte Datei (Zeilen) | 972 (`crates/store/tests/store.rs`) | 883 (`crates/export/src/jpeg.rs`, unverändert). Die Tests der Ablage haben jetzt 842 Zeilen. Die nicht geprüfte `bridge.rs` ist von 978 auf 1041 Zeilen gewachsen. |
| längste Funktion (Zeilen) | 131 (Testhilfe `plan` in `store/tests/store.rs`), im Programmcode `apply` mit 128 | 125 (dieselbe Testhilfe), im Programmcode `apply` mit 120 |

## Funde

### 1. Die offenen Fragen zum Abgleich werden neben dem Register als eigene Liste gehalten, und das Entfernen einer Szene vergisst sie
Manche Einträge des Registers kann der Abgleich keiner Szene des Buchs sicher zuordnen. Für jeden solchen Eintrag stellt er eine Frage, welche Szene es ist. Die Sitzung rechnet diese Fragen nicht bei Bedarf aus dem Register und dem Buch aus. Sie hält sie als eigene Liste und frischt sie an acht Stellen von Hand auf: beim Öffnen, Speichern, Speichern des Buchs, Wiederherstellen, bei neuem Buchtext, bei Rückgängig, Wiederholen und beim Beantworten einer Frage. Bei einer Änderung am Projekt über die Szenenverwaltung (Szene anlegen, entfernen, umbenennen) frischt sie sie nicht auf, außer eine Umbenennung ändert dabei den Buchtext.

Ich habe das mit einem kleinen Programm gegen die Sitzung geprüft. Ein Projekt hat einen Eintrag „9 INT. HOF – NACHT“, und das Buch hat zwei Szenen mit diesem Kopf. Daraus entsteht eine offene Frage. Wird der Eintrag dann entfernt, ist das Register leer. Die Sitzung meldet aber weiter eine offene Frage für den Eintrag, den es nicht mehr gibt (`pending: 1`). Die Statuszeile zeigt das an. Eine Antwort auf diese Frage scheitert.

Was es kostet: Jede neue Stelle, die Register oder Buch ändert, muss daran denken, die Fragen aufzufrischen. Vergisst sie es, fällt das nicht beim Bauen auf, sondern erst beim Benutzer.

Vorschlag: Die Fragen an einer Stelle auffrischen, über die jede Änderung läuft (dort, wo die Sitzung einen neuen Stand übernimmt). Die Alternative ist, sie nur bei Bedarf auszurechnen. Zuerst kommt ein Test für das Entfernen dazu.

Aufwand klein. Risiko gering: Es gibt `crates/services/tests/book_align.rs` und `crates/services/tests/register.rs`.

Beleg: `crates/services/src/session/book.rs:16`, `crates/services/src/session/book.rs:88`, `crates/services/src/session/file.rs:60`, `crates/services/src/session/file.rs:215`, `crates/services/src/session/undo.rs:46`, `crates/services/src/session/undo.rs:149`, `crates/services/src/session/scene.rs:117`, `crates/services/src/project_edit.rs:271`.

### 2. `ARCHITECTURE.md` schickt neue Befehle nach `AppHeader.qml`, sie stehen aber seit #216 in eigenen Dateien
Um die Grenze von 800 Zeilen zu halten, hat #216 die Befehle der Kopfleiste in eigene Dateien gelegt: Ausgaben (`ExportCommands.qml`), Abspielen (`PlayCommands.qml`) und Befehle für die nächste Auswahl (`ArmCommands.qml`). Daneben gibt es schon `ObjectCommands.qml`, `PageExports.qml`, `ShotViewCommands.qml`, `ViewCommands.qml` und `ZoomCommands.qml`. `ARCHITECTURE.md` nennt keine dieser Dateien. In drei Zeilen der Tabelle „Where new things go“ steht weiter `AppHeader.qml`: bei neuen Tasten oder Befehlen für die Auswahl, bei neuen Ausgabeformaten und bei neuen Einstellungen der Ansicht.

Die Zeile für Ausgabeformate nennt den Menüeintrag in `AppHeader.qml`, aber nicht die Befehlspalette: Jedes der neun Formate steht auch in `ExportCommands.qml`.

Was es kostet: Wer der Tabelle folgt, sucht in der falschen Datei. Ein neues Format braucht eine Stelle im Fenster, die nicht genannt ist.

Vorschlag: Die drei Zeilen nennen die Befehlsdateien, die Zeile für Ausgabeformate auch `ExportCommands.qml`.

Aufwand klein, nur Text. Kein Risiko.

Beleg: `ARCHITECTURE.md:90`, `ARCHITECTURE.md:93`, `ARCHITECTURE.md:96`, `crates/app/qml/ExportCommands.qml:23`, `crates/app/qml/ArmCommands.qml:14`, `crates/app/qml/AppHeader.qml:59`, `crates/app/qml/AppHeader.qml:164`.

### 3. Welchen Tooltip ein Griff zeigt, entscheidet das Fenster selbst aus dem Text des Auswahl-Schlüssels
Fährt man mit der Maus über einen Griff, etwa die Breite eines Möbels oder die Kante einer Tür, zeigt der Plan einen Tooltip. Diesen Text sucht `PlanArea.qml` selbst aus. Es zerlegt dafür den Schlüssel des Griffs (z. B. `objectgrip:o:width`) und kennt dazu die Namen der Griffarten, die schon in `plan/key.rs` stehen. Die Brücke sagt für diese Griffe ausdrücklich „kein Tooltip“. Auf dieselbe Weise erkennt das Fenster auch Knicke von Wegen und die Griffe, für die die Pfeiltasten gelten.

`ARCHITECTURE.md` sagt für einen neuen Griff: in der Brücke den Zweig mit „kein Tooltip“ eintragen. Dass der Tooltip in `PlanArea.qml` entsteht, steht dort nicht. Nach den Regeln der Datei soll das Fenster so etwas auch nicht selbst entscheiden.

Was es kostet: Ein neuer Griff, der nach der Tabelle gebaut wird, bekommt still keinen Tooltip, während alle anderen einen haben. Ändert sich die Schreibweise eines Schlüssels, bricht der Tooltip, ohne dass ein Rust-Test es merkt.

Vorschlag: Die Brücke liefert für Griffe den Text-Schlüssel des Tooltips wie für die anderen Elemente. `PlanArea.qml` zeigt ihn nur noch an.

Aufwand klein. Risiko gering: Es gibt die Tests in `crates/app/src/bridge/hints.rs` und `crates/app/tests/qml/tst_planarea.qml`.

Beleg: `crates/app/qml/PlanArea.qml:711`, `crates/app/qml/PlanArea.qml:706`, `crates/app/qml/PlanArea.qml:681`, `crates/app/src/bridge/hints.rs:44`, `crates/app/src/plan/key.rs:33`, `ARCHITECTURE.md:86`.

### 4. Die Frage „Welche Szene des Buchs ist das?“ ist im Terminal und im Fenster je für sich gebaut, mit doppelten Texten
Beide Oberflächen stellen die offenen Fragen aus Fund 1. Jede baut selbst die Liste der Antworten: erst die möglichen Szenen, dann als letzte Zeile „Nicht im Buch“. Das Terminal erkennt diese letzte Zeile daran, dass ihr keine Szene mehr entspricht. Das Fenster gibt ihr die Nummer −1, die die App wieder in „Nicht im Buch“ übersetzt.

Die drei Texte des Dialogs stehen im Katalog zweimal, wortgleich: Titel, Frage und „Nicht im Buch“, einmal als `book.*` und einmal als `tui.book.*`, auf Deutsch und auf Englisch.

`ARCHITECTURE.md` sagt in der Zeile für den Abgleich, dass Fenster und Terminal nur die Statuszeile betreffen. Die Dialoge und die Übersetzung der Antwort nennt die Zeile nicht.

Was es kostet: Eine neue Antwort, etwa „später entscheiden“ für einzelne Einträge, braucht Änderungen in beiden Oberflächen und in der App-Übersetzung. Eine andere Formulierung der Frage braucht vier Einträge im Katalog statt zwei.

Vorschlag: Die Sitzung gibt die Antworten einer Frage als fertige Liste heraus, mit „Nicht im Buch“ als eigenem Eintrag. Beide Oberflächen zeigen sie nur an. Die doppelten Texte fallen weg, und die Zeile in `ARCHITECTURE.md` nennt die Dialoge.

Aufwand klein bis mittel. Risiko gering: Es gibt `crates/tui/src/tests/book.rs`, `crates/app/tests/qml/tst_bookdialog.qml` und die Tests in `crates/app/src/manage.rs`.

Beleg: `crates/tui/src/app/book.rs:37`, `crates/tui/src/app/book.rs:56`, `crates/app/src/manage.rs:61`, `crates/app/src/manage.rs:65`, `crates/app/qml/BookDialog.qml:27`, `crates/services/i18n/de.json:429`, `crates/services/i18n/de.json:707`, `ARCHITECTURE.md:84`.

### 5. Zwei kleine Doppelungen beim Register: die freie Kennung eines neuen Eintrags, der Dateiname des Drehbuchs
Ein neuer Eintrag im Register bekommt eine zufällige Kennung. Sie darf sich von keiner vorhandenen nur durch Groß- und Kleinschreibung unterscheiden, sonst würden sich auf manchen Rechnern die Dateien überschreiben. Diese Suche nach einer freien Kennung steht zweimal in der Sitzung: beim Abgleich mit dem Buch und beim Anlegen einer Szene von Hand.

Der Dateiname `drehbuch.fountain` steht in der Ablage und noch einmal in der Sitzung. Die Sitzung braucht ihn nur, um bei Konfliktmarken von Git im Drehbuch den Pfad in der Fehlermeldung zu nennen.

Was es kostet: Eine geänderte Regel für Kennungen gilt sonst nur auf einem der beiden Wege. Ein umbenanntes Drehbuch zeigt in dieser Fehlermeldung den alten Namen.

Vorschlag: Eine Funktion für die freie Kennung, die beide Wege benutzen. Die Ablage nennt den Pfad des Drehbuchs (oder meldet den Konflikt selbst), statt dass die Sitzung ihn wiederholt.

Aufwand klein. Risiko gering: Es gibt `crates/services/tests/book_align.rs`, `crates/services/tests/manage.rs` und `crates/store/tests/register.rs`.

Beleg: `crates/services/src/book.rs:426`, `crates/services/src/project_edit.rs:566`, `crates/services/src/load.rs:10`, `crates/store/src/lib.rs:60`.

### 6. Bestandsliste
**Gewachsen:**
- `name` in `field.rs` hat jetzt 45 statt 44 Zweige (#215). Der Commit nennt dafür keinen Grund. Es ist aber eine flache Liste der Feldnamen mit einem Zweig je Feld und in Ordnung.

**Kann sich teilen:**
- `script/src/assist.rs` ist durch #226 von 624 auf 769 Zeilen gewachsen. Die Datei hat zwei Aufgaben: Vorschläge beim Tippen und den Ablauf von Enter und Tab (`Slot`, `enter_step`, `tab_step`). Das ist eine natürliche Naht. Der Ablauf kann in eine eigene Datei, bevor die Datei die Grenze erreicht.

**An der Grenze von 800 Zeilen:**
- `stellprobe/src/motion.rs` mit 797 (vorher 770),
- `model/src/plan.rs` mit 772 (vorher 750),
- `services/src/plan_edit.rs` mit 763 (vorher 743),
- `export/tests/export.rs` mit 790, `services/tests/session.rs` und `app/src/bridge/view.rs` mit je 788, `PlanArea.qml` mit 778.

Bei den ersten drei kam der Zuwachs von den Wegen der Requisiten (#214).

**In Ordnung:**
- die flachen Listen (`field.rs` `name`, `plan/key.rs` `key`),
- die alten Einträge der Ausgabe (`jpeg.rs`, `layout.rs`, `scan.rs`),
- `apply`, das weiter schrumpft.

Die nicht geprüfte `bridge.rs` wächst weiter (1041 Zeilen), wie in `ARCHITECTURE.md` vorgesehen.

Beleg: `structure.baseline`, `crates/script/src/assist.rs:54`, `crates/script/src/assist.rs:115`, `crates/stellprobe/src/motion.rs:1`.

## Seit dem letzten Audit
Seit dem 14. Audit wurden 19 PRs gemergt: #202, #208, #213 bis #229.

14. Audit (alle acht Funde vom Eigentümer zum Beheben gewählt):
- **Fund 1 (Titelseite mit eigener Regel und vier Stellen): behoben (#221).** Die Sitzung liest die Felder über `title_fields`, und das Terminal ordnet die Beschriftungen nach dem Namen der Felder zu. Beleg: `crates/services/src/script_pages.rs:124`, `crates/tui/src/i18n.rs:36`.
- **Fund 2 (auskommentierte Teile zweimal bestimmt): größtenteils behoben (#222).** Die Zeichen stehen nur noch im Leser. Ob eine Zeile innerhalb eines Kommentars beginnt, übernimmt der Editor jetzt vom Leser. Innerhalb einer Zeile entscheidet er aber weiter selbst, und dort lesen beide noch verschieden. Ich habe es geprüft: `Vorher \/* weg */ danach.` Der Leser schneidet „/* weg */“ heraus, und das PDF zeigt `Vorher \ danach.`. Der Editor zeigt denselben Teil kursiv statt grau. Beleg: `crates/script/src/inline.rs:72`, `crates/script/src/inline.rs:103`, `crates/script/src/parse.rs:150`.
- **Fund 3 (gebundenes Etikett an vier Stellen): behoben (#225).** Ein Typ `Binding` im Datenmodell. Die Palette fragt die Brücke. Beleg: `crates/model/src/binding.rs:11`, `crates/app/qml/ObjectCommands.qml:22`.
- **Fund 4 (Papiergrößen und Textbreite doppelt): behoben (#228).** Die Ausgabe hat eine Tabelle der Papiermaße, und die Textbreite kommt mit dem Blatt. Es bleibt eine Kleinigkeit: Die Sitzung übersetzt das Papier des Drehbuchs weiter an zwei Stellen. Beleg: `crates/services/src/script_pages.rs:96`, `crates/services/src/script_pages.rs:239`.
- **Fund 5 (Enter und Tab nur im Terminal): größtenteils behoben (#226).** Die drei Regeln stehen jetzt im Drehbuch-Baustein. Es bleibt ein Rest: Das Terminal übersetzt seine Zeilenarten weiter selbst in die Rollen des Ablaufs. Die gleichwertige Funktion `Slot::of` rufen weiter nur Tests auf. Das Zeichen „)“ steht in `assist.rs` zweimal als Konstante und im Terminal ein drittes Mal. Beleg: `crates/tui/src/fountain.rs:35`, `crates/script/src/assist.rs:74`, `crates/script/src/assist.rs:52`, `crates/script/src/assist.rs:114`, `crates/tui/src/app.rs:34`.
- **Fund 6 („(MORE)“ und „(CONT'D)“ an zwei Stellen): behoben (#227).** Beleg: `crates/script/src/pages/flow.rs:17`.
- **Fund 7 (`ARCHITECTURE.md` nennt nicht alle Stellen eines Drehbuch-Elements): behoben (#229).** Beleg: `ARCHITECTURE.md:100`.
- **Fund 8 (Bestandsliste): behoben (#223 und #216).** Der Eintrag `write` ist weg. `AppHeader.qml` ist von 799 auf 655 Zeilen geschrumpft. Die Testdateien und `scene.rs` sind geteilt.

13. Audit:
- **Fund 3 (Karten suchen ihr Element mit kopiertem Block, siebenmal): offen.** Beleg: `crates/app/qml/ImageCard.qml:16` und die sechs anderen Karten.
- **Funde 5 und 6: behoben (#211).**

12. Audit:
- **Fund 5 (Kopieren über einen gemeinsamen Ablauf): vom Eigentümer nicht gewählt.** Er bleibt stehen und wird nicht neu gemeldet.

Aus dem 10., 8., 7., 6., 5., 4. und 3. Audit sind weiter alle Funde offen, die das 14. Audit dort als offen führt. Beim 6. Audit, Fund 1 (Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers) hat sich am Beleg nichts geändert: `crates/tui/src/fountain.rs:201`, `crates/script/src/parse.rs:483`.

Geprüft:
- Die Tests der Bausteine Drehbuch, Terminal, Sitzung, Ablage, Datenmodell und Ausgabe sind in diesem Audit grün gelaufen (`cargo test -p klappe-script -p klappe-tui -p klappe-services -p klappe-store -p klappe-model -p klappe-export`).
- Die veraltete offene Frage in Fund 1 und das verschiedene Lesen von `\/*` beim 14. Audit, Fund 2, habe ich mit kleinen Programmen gegen Sitzung und Drehbuch-Baustein nachgewiesen. Sie lagen außerhalb des Repos und sind gelöscht.
- Die Strukturprüfung meldet 558 Dateien, 0 Verstöße und 26 Einträge in der Bestandsliste.

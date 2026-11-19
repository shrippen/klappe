# Code-Audit Klappe, 18.11.2026, Stand 324d1dbe

## Urteil
Das Repo ist wartbar, aber etwas schlechter als beim 16. Audit. Die Bestandsliste ist kleiner geworden (26 auf 24 Einträge, ein weiterer kann gestrichen werden). Die drei neuen Bausteine dieses Abschnitts, die Spielorte (#256), das Video über ffmpeg (#246) und das Ein- und Ausblenden der Figuren (#255), sitzen im richtigen Baustein, und `ARCHITECTURE.md` nennt sie. Jeder bringt aber eine Doppelung mit: Das Video baut nach, wie die Sitzung den Plan zusammenstellt (Fund 3). Wie blass etwas gezeichnet wird, rechnen Fenster und Ausgabe jeweils selbst aus (Fund 2). Die Spielorte stecken in den Projektdaten, obwohl ihre Datei die einzige Quelle sein soll (Fund 4). Am dringendsten ist Fund 1: Die Fragen zu den Spielorten formuliert heute das Terminal selbst. Der noch nicht gemergte PR #258 (Zweig `gui-orte`) baut dieselben Formulierungen im Fenster ein zweites Mal.

## Kennzahlen
| | letztes Audit | jetzt |
|---|---|---|
| Quelldateien | 559 | 586 (gezählt wie die Strukturprüfung, ohne die Kopie von Kante) |
| Einträge in der Bestandsliste | 26 | 24. Gestrichen sind `figure_items` (#255) und `item_prims` (geteilt). `save` in `store/src/lib.rs` heißt jetzt `save_with`, mit demselben Wert. Ein weiterer Eintrag (`load` in `store/src/lib.rs`) liegt nicht mehr über der Grenze; nach `--update` bleiben 23. |
| größte Datei (Zeilen) | 883 (`crates/export/src/jpeg.rs`) | 883, dieselbe Datei. Die nicht geprüfte `bridge.rs` ist von 1041 auf 1067 Zeilen gewachsen. |
| längste Funktion (Zeilen) | 125 (Testhilfe `plan` in `store/tests/store.rs`), im Programmcode `apply` mit 120 | unverändert 125 und 120 |

## Funde

### 1. Die Fragen zum Buch formuliert jede Oberfläche selbst, und für die Spielorte entsteht das gerade ein zweites Mal
Zum Buch gibt es jetzt zwei Arten von Fragen: welche Szene ein Eintrag des Registers ist, und welcher Spielort ein Ort aus einem Szenenkopf ist. Die Sitzung liefert für beide die möglichen Antworten, aber nicht den Text dazu. Den Wortlaut setzt das Terminal selbst zusammen: Es wählt je nach Art der Antwort den passenden Text („Neuer Spielort HOF“, „Schreibweise von SAAL“, „SAAL heißt jetzt HOF“) und setzt die Namen ein. Bei den Szenenfragen ist das schon heute doppelt, denn Terminal und Fenster bauen die Frage und die Zeile „nicht im Buch“ jeweils selbst. Der noch offene PR #258 schreibt jetzt auch die Auswahl der Spielort-Texte im Fenster ein zweites Mal. Dazu verzweigt das Terminal an drei Stellen von Hand zwischen Szenenfrage und Ortsfrage: bei der Zahl der Zeilen, beim Antworten und beim Zeichnen.

Was es kostet: Ändert sich ein Wortlaut oder kommt eine neue Antwort dazu (etwa „ist kein Spielort“), braucht es Änderungen an zwei Oberflächen, und sie können auseinanderlaufen. Eine dritte Art von Frage braucht in jeder Oberfläche neue Verzweigungen. `ARCHITECTURE.md` sagt dagegen, die Oberflächen zeigen die Zeilen nur an. Die Statuszeile macht es schon richtig: Ihren Text liefert die Sitzung fertig (`book_report_text`).

Vorschlag: Die Sitzung gibt eine einzige Liste offener Fragen heraus, jede mit fertigem Titel, Frage und Antwortzeilen in der gewählten Sprache, und nimmt die Antwort über einen einzigen Aufruf mit der Position entgegen. Am besten vor oder zusammen mit #258, damit die zweite Kopie gar nicht erst entsteht.

Aufwand mittel. Risiko gering: Es gibt `crates/services/tests/book_places.rs`, `crates/tui/src/tests/places.rs`, `crates/tui/src/tests/book.rs` und `crates/app/tests/qml/tst_bookdialog.qml`.

Beleg: `crates/tui/src/i18n.rs:203`, `crates/tui/src/i18n.rs:211`, `crates/tui/src/i18n.rs:220`, `crates/tui/src/ui_overlay.rs:268`, `crates/tui/src/ui_overlay.rs:275`, `crates/tui/src/ui_overlay.rs:288`, `crates/tui/src/app/book.rs:68`, `crates/tui/src/app/book.rs:83`, `crates/app/qml/BookDialog.qml:29`, `crates/app/qml/BookDialog.qml:65`, auf dem Zweig `gui-orte`: `crates/app/qml/BookDialog.qml:29`, `crates/app/qml/BookDialog.qml:38`; `crates/services/src/session/book.rs:32`, `crates/services/src/session/places.rs:32`, `ARCHITECTURE.md:85`.

### 2. Wie blass etwas gezeichnet wird, rechnen Fenster und Ausgabe jeweils selbst aus
Ein Element kann aus zwei Gründen blasser erscheinen: Seine Ebene ist abgeblendet, oder eine Figur blendet gerade ein oder aus (#255). Die Sitzung liefert nur den ersten Wert. Das Malen mit dem zweiten erledigen Fenster und Ausgabe jeweils selbst, im Plan an zwei Stellen und im Bild der 3D-Ansicht noch einmal an zwei.

Was es kostet: `ARCHITECTURE.md` verlangt, dass das Aussehen eines Elements einmal festgelegt wird, damit der Ausdruck dem Bildschirm gleicht. Kommt ein dritter Grund zum Abblenden dazu (etwa „nicht in dieser Einstellung“), müssen ihn vier Stellen in zwei Bausteinen lernen. Fehlt er an einer, weicht der Ausdruck still vom Bildschirm ab.

Vorschlag: Die Sitzung liefert für jedes Element des Plans die fertige Stärke, Ebene und Einblenden zusammen (`item_fades`, `fades_of`), und Fenster und Ausgabe nehmen nur noch diese Zahl. Für das Bild der 3D-Ansicht gilt dasselbe, wenn das Bild seine Stärken schon mit dem Einblenden zusammen trägt.

Aufwand klein. Risiko gering: Es gibt `crates/export/tests/fade.rs`, `crates/services/tests/entrance.rs` und `crates/stellprobe/tests/picture.rs`.

Beleg: `crates/services/src/session/fade.rs:59`, `crates/services/src/session/pages.rs:147`, `crates/app/src/plan.rs:221`, `crates/export/src/layout.rs:184`, `crates/export/src/picture_png.rs:60`, `crates/app/src/plan/picture.rs:122`, `ARCHITECTURE.md:70`.

### 3. Das Video baut nach, wie die Sitzung den Plan zusammenstellt
Damit das Fenster beim Schreiben des Videos nicht stehen bleibt, nimmt das Video eine Kopie mit: das Projekt, die gewählte Szene und den Plan, die ausgeblendeten Ebenen und Spuren, die abgeblendeten Ebenen. Die Felder kopiert es einzeln. Aus der Kopie stellt es jedes Bild selbst zusammen. Dieser Weg steht damit zweimal im Repo: Szene und Plan suchen, den Plan zeichnen, Ausgeblendetes weglassen, Bilddateien lesen, Bildgrößen merken. Einmal steht er in der Sitzung, einmal im Video. #246 hat die Teilschritte schon in gemeinsame Funktionen gezogen, den Ablauf aber nicht.

Was es kostet: Eine neue Ansichtseinstellung (die Zeile „New view setting“ in `ARCHITECTURE.md`) muss auch ins Video, sonst zeigt das Video etwas anderes als der Bildschirm. Die Tabelle nennt das Video dort nicht, und der Compiler meldet nichts.

Vorschlag: Was eine Ansicht des Plans ausmacht (Projekt, Szene und Plan, Ein- und Abblenden), wird ein eigener Wert. Die Sitzung hält ihn, das Video nimmt eine Kopie davon, und beide stellen den Plan über dieselbe Funktion zusammen. Bis dahin nennt die Zeile in `ARCHITECTURE.md` das Video.

Aufwand mittel. Risiko gering: Es gibt `crates/services/tests/video.rs` und die Tests der Ebenen und des Abblendens.

Beleg: `crates/services/src/session/video.rs:39`, `crates/services/src/session/video.rs:84`, `crates/services/src/session/video.rs:68`, `crates/services/src/session/video.rs:93`, `crates/services/src/session/video.rs:193`, `crates/services/src/session/pick.rs:35`, `crates/services/src/session/pick.rs:57`, `crates/services/src/session/export.rs:226`, `ARCHITECTURE.md:97`.

### 4. Die Spielorte stecken in den Projektdaten, obwohl ihre Datei die einzige Quelle sein soll
Eine Antwort zu einem Spielort wird sofort in `orte.toml` geschrieben, ausgehend von der Datei auf der Platte. Rückgängig machen gibt es dafür nicht, und Speichern lässt die Datei in Ruhe. Trotzdem sind die Spielorte ein Teil der Projektdaten. Deshalb wandern sie in jede Rückgängig-Stufe und in jede Sicherung mit, und die Sitzung muss sie an drei Stellen von Hand wieder richtigstellen. Beim Rückgängigmachen behält sie die aktuellen Spielorte. Beim Wiederherstellen einer Sicherung liest sie die Datei neu. Beim Speichern nimmt sie eine eigene Speicherart, die die Datei auslässt. Die Sicherung schreibt dabei eine `orte.toml`, die beim Wiederherstellen nie gelesen wird. Den Ablageteil (`store/`) betrifft das auch: Er hat dafür einen Schalter zwischen zwei Arten zu speichern bekommen.

Was es kostet: Jeder neue Weg, auf dem die Sitzung ihre Projektdaten ersetzt, muss an die Spielorte denken. Vergisst er sie, zeigt die Sitzung alte Spielorte und falsche offene Fragen, und kein Test fällt um, der nicht genau diesen Weg prüft. Dasselbe Muster gibt es schon für das Papier der Ausgabe.

Vorschlag: Die Sitzung hält die Spielorte neben den Projektdaten statt darin und liest und schreibt sie nur über `load_places` und `save_places`. Dann entfallen die drei Stellen, die zweite Speicherart und die ungelesene Datei in der Sicherung. Die Beispielprojekte schreiben ihre Spielorte dann mit einem eigenen Aufruf.

Aufwand mittel. Risiko gering: Es gibt `crates/store/tests/places.rs` und `crates/services/tests/book_places.rs`.

Beleg: `crates/model/src/project.rs:20`, `crates/services/src/session/undo.rs:188`, `crates/services/src/session/undo.rs:191`, `crates/services/src/session/file.rs:114`, `crates/services/src/session/file.rs:162`, `crates/store/src/lib.rs:236`, `crates/store/src/lib.rs:242`, `crates/store/src/lib.rs:336`, `crates/store/src/lib.rs:396`, `crates/store/src/lib.rs:419`.

### 5. Das Terminal zeigt Fehler auf zwei Arten, meist als englischen Entwicklertext
Das Fenster übersetzt jeden Fehler der Sitzung über eine gemeinsame Funktion in einen Satz auf Deutsch oder Englisch. Das Terminal macht das nur an einer Stelle, der neuen Antwort zu einem Spielort. An den sieben anderen Stellen zeigt es den rohen englischen Text des Fehlers, auch in der deutschen Oberfläche: beim Speichern, beim Sichern, bei der Antwort zu einer Szene, bei der Sicherung und beim neuen PDF des Drehbuchs (#241).

Was es kostet: Ein Fehler, den das Fenster gut erklärt, erscheint im Terminal als „no open decision for scene …“. Wer eine neue Fehlermeldung schreibt, sieht nicht, welcher Weg gilt.

Vorschlag: Das Terminal nimmt überall dieselbe Übersetzung wie das Fenster.

Aufwand klein. Risiko gering: Der Wortlaut der Fehler ist in `crates/services/src/error_text.rs` getestet.

Beleg: `crates/tui/src/app/book.rs:92`, `crates/tui/src/app/book.rs:115`, `crates/tui/src/app/book.rs:172`, `crates/tui/src/app.rs:585`, `crates/tui/src/app/backup.rs:63`, `crates/tui/src/app/backup.rs:75`, `crates/tui/src/app/backup.rs:99`, `crates/tui/src/app/pdf.rs:126`, `crates/app/src/bridge/book.rs:21`.

### 6. Ob zwei Figuren gleich heißen und welcher Name frei ist, steht an mehreren Stellen
Zwei Figuren dürfen nicht gleich heißen, verglichen in Großbuchstaben wie die Namen über den Repliken. Diese Regel steht viermal im Repo, jedes Mal als eigener Vergleich. Den nächsten freien Namen („Anna 2“, „Anna 3“) sucht das Kopieren einer Figur (#245) mit einer neuen Funktion. Fast dieselbe Funktion gibt es schon beim Einsetzen einer Vorlage. Für die Spielorte hat #256 die Regel dagegen einmal neben den Typ geschrieben (`same_place_name`).

Was es kostet: Ändert sich die Regel (etwa Leerzeichen oder „ß“ wie bei den Spielorten), gilt sie sonst nur an einer Stelle. Zwei Figuren, die die eine Stelle für verschieden hält und die andere für gleich, verschmelzen im Drehbuch.

Vorschlag: Eine Funktion für „gleicher Figurenname“ neben dem Typ der Figur, wie bei den Spielorten, und eine einzige Suche nach dem nächsten freien Namen, die Kopieren und Vorlagen beide nutzen.

Aufwand klein. Risiko gering: Es gibt Tests in `crates/services/src/project_edit/copy.rs`, `crates/services/tests/copies.rs` und die Tests der Vorlagen.

Beleg: `crates/services/src/project_edit.rs:353`, `crates/services/src/project_edit/copy.rs:56`, `crates/services/src/project_edit/templates.rs:166`, `crates/services/src/script_refs.rs:138`, `crates/model/src/place.rs:19`.

### 7. Der Spielort hat drei englische Namen, und „location“ meint zwei Dinge
Im Code heißt der Spielort `Place`, in den Kommentaren und in `ARCHITECTURE.md` „play location“, in der englischen Oberfläche und in der Begriffsliste von #257 „scripted location“. „Location“ allein ist im Code der Drehort, in den neuen Kommentaren aber auch der Ort, den ein Szenenkopf nennt. Die Zeile für die Projektdaten in `ARCHITECTURE.md` nennt den Spielort nicht.

Was es kostet: Wer nach dem Begriff der Oberfläche sucht, findet den Code nicht. Wer „location“ liest, muss raten, ob Drehort oder Spielort gemeint ist.

Vorschlag: Ein englischer Name für den Spielort in Code, Kommentaren und `ARCHITECTURE.md`, so wie ihn die Begriffsliste festlegt. Dazu nennt die Zeile für `model/` den Spielort.

Aufwand klein (nur Text und Namen). Risiko gering.

Beleg: `crates/model/src/place.rs:5`, `crates/services/src/places.rs:9`, `crates/services/src/session/places.rs:1`, `planung/BEGRIFFE.md:27`, `ARCHITECTURE.md:29`, `ARCHITECTURE.md:85`.

### 8. Der Sprung zwischen Buch und Plan beschreibt seine Verweise ein zweites Mal
Die Sitzung beschreibt jeden Verweis einer Drehbuchzeile auf den Plan (Mark oder Einstellung), mitsamt dem Zeitpunkt. Die Brücke zum Fenster (#254) hat dafür eine eigene Liste. Sie übersetzt die Art des Verweises über ausgeschriebene Wörter („mark“, „shot“), die von Hand zu den Namen in der Sitzung passen müssen. Den Zeitpunkt einer Mark sucht sie im Plan selbst, obwohl der Verweis ihn schon trägt.

Was es kostet: Eine dritte Art von Verweis (etwa ein Lichtwechsel) braucht Änderungen an vier Stellen der Brücke. Weicht ein Wort ab, springt der Klick still nicht.

Vorschlag: Die Sitzung nimmt den Verweis entgegen und erledigt den Sprung selbst. Die Brücke reicht ihn nur weiter.

Aufwand klein. Risiko gering: Es gibt `crates/app/tests/controller/script_jump.qml` und `crates/app/tests/qml/tst_scriptpanel.qml`.

Beleg: `crates/app/src/bridge/jump.rs:14`, `crates/app/src/bridge/jump.rs:20`, `crates/app/src/bridge/jump.rs:72`, `crates/services/src/views.rs:139`, `crates/services/src/views.rs:155`.

### 9. Bestandsliste
**Kleiner geworden:** `figure_items` (#255) und `item_prims` (die Beschriftung steht jetzt in `layout/swatch.rs`) sind aus der Liste. `load` in `store/src/lib.rs` liegt nicht mehr über der Grenze und kann mit `--update` gestrichen werden.

**Umbenannt, nicht erhöht:** `save` in `store/src/lib.rs` heißt jetzt `save_with` (#256), mit unverändertem Wert 18.

**An der Grenze von 800 Zeilen:** `stellprobe/src/motion.rs` steht genau bei 800 (vorher 797). Die natürliche Naht sind die Tests ab Zeile 402. Sie können in eine eigene Datei, wie es `display/tests.rs` schon vormacht. `store/src/lib.rs` ist durch die Spielorte von 743 auf 799 Zeilen gewachsen. Natürliche Nähte sind die Prüfungen beim Laden (`check_*` ab Zeile 492) und die Sicherung (ab Zeile 389). Werden die Prüfungen verschoben, muss die Zeile „New kind of element“ in `ARCHITECTURE.md` mit, die sie in `lib.rs` verortet. Behebt jemand Fund 4, schrumpft die Datei ohnehin. Ebenfalls nahe der Grenze: `services/tests/session.rs` mit 795, `export/tests/export.rs` mit 792, `app/src/bridge/view.rs` mit 788, `PlanArea.qml` mit 774 (vorher 761), `model/src/plan.rs` mit 772, `stellprobe/src/display/tests.rs` mit 771, `services/src/plan_edit.rs` mit 765.

**In Ordnung:** die flachen Listen (`field.rs` `name`, `plan/key.rs` `key`), die alten Einträge der Ausgabe (`jpeg.rs`, `scan.rs`, `deflate.rs`, `inflate.rs`) und `apply`.

Beleg: `structure.baseline`, `crates/stellprobe/src/motion.rs:402`, `crates/store/src/lib.rs:389`, `crates/store/src/lib.rs:492`, `ARCHITECTURE.md:86`.

## Seit dem letzten Audit
Seit dem 16. Audit wurden zehn PRs gemergt: #241, #243, #245, #246, #249, #251, #254, #255, #256 und #257 (dazu #244, der das 16. Audit selbst ablegt). #258 (Spielort-Fragen im Fenster) liegt erst auf dem Zweig `gui-orte` und ist nicht in diesem Stand. Er betrifft Fund 1.

16. Audit (der Eigentümer hat am 18.11.2026 entschieden, die Funde 1, 2 und 3 beheben zu lassen; noch nicht behoben):
- **Fund 1 (Prüfung „hat sich das Register geändert“ mit einer Feldliste von Hand): offen, zur Behebung gewählt.** Unverändert. Beleg: `crates/services/src/session/book.rs:147`.
- **Fund 2 (das Terminal führt selbst Buch über Strg+Z für Antworten; `ARCHITECTURE.md` nennt das nicht): offen, zur Behebung gewählt.** Die Antworten zu Spielorten sind kein Schritt von Strg+Z. Die Buchführung muss sie also ebenfalls auslassen, was sie heute tut. Die Zeile in `ARCHITECTURE.md` nennt die Stelle weiter nicht. Beleg: `crates/tui/src/app/book.rs:97`, `crates/tui/src/app/book.rs:106`, `crates/tui/src/app/book.rs:131`, `ARCHITECTURE.md:85`.
- **Fund 3 (deutsche Szenenköpfe, Großschreibung, Sprachübersetzung doppelt): offen, zur Behebung gewählt.** #249 hat die Großschreibung des Terminals in den Drehbuch-Baustein verlegt, mit einer eigenen Konstante für „ß“. Damit steht die Ausnahme für „ß“ jetzt dreimal im Drehbuch-Baustein. Der Namensvergleich der Spielorte hat dazu eine eigene Regel für „ß“. Beleg: `crates/script/src/parse.rs:17`, `crates/script/src/parse.rs:539`, `crates/script/src/parse.rs:546`, `crates/script/src/flow.rs:156`, `crates/model/src/place.rs:27`, `crates/services/src/session/file.rs:90`.
- **Fund 4 (Bestandsliste):** Die Tests des Abgleichs (`services/tests/book_align.rs`) sind bei 708 Zeilen geblieben.

14. Audit, Reste:
- **Fund 2 (auskommentierte Teile, Regel zweimal): offen.** Beleg: `crates/script/src/parse.rs:203`, `crates/script/src/inline.rs:104`.
- **Fund 4 (Papier des Drehbuchs an zwei Stellen übersetzt): offen.** Beleg: `crates/services/src/script_pages.rs:98`, `crates/services/src/script_pages.rs:240`.
- **Fund 5 (Enter und Tab): Rest offen.** `Slot::of` wird jetzt auch außerhalb der Tests aufgerufen (`flow.rs`); dieser Teil ist erledigt. Das Terminal übersetzt seine Zeilenarten weiter selbst. Beleg: `crates/tui/src/fountain.rs:35`, `crates/script/src/flow.rs:220`.

13. Audit:
- **Fund 3 (Karten suchen ihr Element mit kopiertem Block, siebenmal): offen.** Beleg: `crates/app/qml/ImageCard.qml:21`, `crates/app/qml/CameraCard.qml:28`, `crates/app/qml/ObjectCard.qml:30` und die anderen Karten.

12. Audit:
- **Fund 5 (Kopieren über einen gemeinsamen Ablauf): vom Eigentümer nicht gewählt.** Er bleibt stehen und wird nicht neu gemeldet.

Aus dem 10., 8., 7., 6., 5., 4. und 3. Audit sind weiter alle Funde offen, die das 16. Audit dort als offen führt. Beim Fund „Die Großschreibung beim Tippen kopiert Regeln des Drehbuchlesers in das Terminalprogramm“ ist durch #249 ein Teil erledigt: Die Umwandlung in Großbuchstaben steht jetzt im Drehbuch-Baustein, und das Terminal ruft sie auf. Welcher Teil einer Zeile großgeschrieben wird, entscheidet das Terminal weiter selbst. Beleg: `crates/script/src/flow.rs:163`, `crates/tui/src/fountain.rs:176`, `crates/tui/src/fountain.rs:186`.

# Klappe: Aufgaben für QA-Runden

Aufgaben für die Personas aus [`PERSONAS.md`](PERSONAS.md), Verfahren im Skill `qa-personas`.
Jede Aufgabe nennt ein Ziel in den Worten der Persona, nie den Weg und keinen Namen eines
Bedienelements. Der Tester bekommt nur den Text unter „Aufgabe“; „Endzustand“ und „Bezug“
prüft der Overseer. Ausgang je Aufgabe: erreicht, mit Umweg, nicht gefunden, nicht prüfbar.

Start: `scripts/qa-session.sh start` öffnet die Startseite; das Beispielprojekt (Szene 14 aus
„Nachtwache“, de oder en) liegt dort als Kachel. Mit `--open` öffnet die Sitzung es direkt.
Aufgaben mit Dateidialog sind „nicht prüfbar“, bis der Test des Sitzungsskripts zeigt, dass der
Dialog in der Sitzung erscheint; Ausgaben prüft der Overseer dann über `klappe-cli`.

Ausgangslage im Beispielprojekt (für den Overseer): Ilva hat einen Mark und geht bei 34 s ab,
der Wärter erscheint bei 3 s und hat einen Knick auf dem Weg zu Mark 2; Kameras A (zwei Marks)
und B (Super 35, 50 mm, auf Ilva); fünf Shots ohne Achsensprung; Achse Ilva–Wärter; drei Lampen;
Set mit Tür, zwei Fenstern, Tisch, Stuhl und Säule. Ein Endzustand nennt nur, was sich
gegenüber dieser Lage ändert.

Eine Runde nimmt höchstens drei Personas mit je vier bis sechs Aufgaben. Neue Aufgaben kommen
dazu, wenn `STAND.md` einen Punkt als fertig oder in erster Fassung führt. Nicht abgedeckt: die
TUI (eigene Runde, wenn gewünscht) und das Hintergrundbild (LH-13; das Beispiel hat kein Bild).

## Abdeckung

| Id | Persona | Aufgabe | Endzustand | Bezug |
|---|---|---|---|---|
| A-01 | P1 | Öffne das Beispielprojekt und schau dir die Szene an, als würdest du sie zum ersten Mal vorbereiten. Lass die Szene einmal von Anfang bis Ende ablaufen. | Projekt offen, Szene bis zum Ende gespielt | LH-60, D-06, D-07 |
| A-02 | P1 | Stell eine zusätzliche Figur in die Szene, die durch die Tür hereinkommt, zum Tisch geht und sich setzt. | neue Figur; erscheint an oder vor der Tür (Auftritt oder Weg von außen); letzter Mark am Tisch „sitzend“ | LH-03, LH-21, LH-22, LH-26 |
| A-03 | P1 | Ilva soll vom Fenster zur Tür gehen, aber im Bogen um den Tisch, nicht quer drüber. | Ilva hat einen neuen Mark an der Tür; ihr Weg dorthin hat einen Knick neben dem Tisch | LH-21 |
| A-04 | P1 | Probier eine zweite Fassung der Auflösung, ohne die erste zu verlieren, und geh danach zur ersten zurück. | zwei Pläne, der erste gezeigt, unverändert | LH-04 |
| A-05 | P2 | Add a third camera for a close-up of the keeper: full-frame body, 85 mm lens. Check what it sees. | Kamera C mit Full Frame, 85 mm, Blick auf den Wärter; Kamerablick zeigt ihn | LH-32, LH-36 |
| A-06 | P2 | When the keeper walks to his second mark, camera A should dolly alongside him at 1.2 m, tilted 10° down. | Kamera-Marks von A im Zeitfenster des Wegs, Gerät Dolly, Höhe 1,2 m, Neigung −10° | LH-33, LH-34 |
| A-07 | P2 | Find out whether any shot breaks the 180° line. Then add a shot from the other side of the line and check again. | erst: keiner; dann neuer Shot, den das Programm als Achsensprung zeigt | LH-40, LH-41, LH-70 |
| A-08 | P3 | Bring die Shots in die Reihenfolge, in der am Drehtag gedreht wird, und prüfe, welche Shots dieselbe Kameraposition haben. | Drehreihenfolge gezeigt, Setups erkennbar | LH-71, LH-72 |
| A-09 | P3 | Gib die Shotliste so aus, dass die Produktion sie in ihrer Tabellenkalkulation öffnen kann. | ODS oder XLSX; CSV ist ein Umweg (ODS und XLSX fehlen laut STAND noch im Menü) | LH-85 |
| A-10 | P3 | Ändere bei einem Shot die Einstellungsgröße und die Beschreibung und stell sicher, dass es nach dem Schließen und Wiederöffnen noch so ist. | Werte nach Neustart erhalten | LH-05, LH-70, LH-71 |
| A-11 | P4 | Stell zwei weitere Lampen in den Plan: eine Stufenlinse mit 2 kW durchs Fenster und ein LED-Panel mit 3200 K auf Ilva. Verschieb dabei nichts anderes. | zwei neue Lampen mit Werten; Figuren, Kameras, Set unverändert | LH-50, LH-51, LH-15 |
| A-12 | P4 | Druck einen Plan, auf dem deine Lampen mit ihren Werten stehen. | PDF mit Lampenliste | LH-80 |
| A-13 | P2 | Later the keeper's second mark moves half a metre. Camera A should move with him without redoing its marks. | A folgt dem Wärter; nach Verschieben des Marks sind die Kamera-Marks mitgewandert | LH-37 |
| A-14 | P2 | Set up one new shot completely without touching the mouse. | neuer Shot mit Kamera, Größe, Objektiv; `actions.log` ohne Klick und Ziehen | NF-06, D-02 |
| A-15 | P5 | Der Raum bekommt eine Nische: eine zusätzliche Wand mit einer Tür, davor ein Sofa. Trag ein, wie hoch der Tisch ist, und miss, wie breit der Durchgang zwischen Tisch und Säule ist. | neue Wand mit Tür, Sofa, Tisch mit Höhe; Messwert genannt | LH-10, LH-11, LH-14, LH-17 |
| A-16 | P5 | Zeichne mit einem Pfeil ein, wo die Requisite den Weg frei halten muss, und schreib eine Notiz für das Team dazu. | Pfeil und Notiz im Plan, beide im PDF | LH-16, LH-19, LH-80 |
| A-17 | P1 | An der Tür sollen zwölf Gäste stehen, ohne zwölf Figuren einzeln hinzustellen. | eine Gruppe mit Zahl 12 an der Tür | LH-23 |
| A-18 | P1 | Setz für ein neues Gespräch eine übliche Schuss-Gegenschuss-Auflösung ein, ohne sie von Hand zu bauen. Mach danach drei Änderungen und nimm zwei davon zurück. | Vorlage eingefügt; eine Änderung bleibt | LH-07, LH-06 |

## Ketten

| Id | Persona | Aufgabe | Endzustand | Bezug |
|---|---|---|---|---|
| K-01 | P1 | Der Wärter heißt jetzt anders. Benenne ihn um und prüfe, ob der neue Name überall steht: im Drehbuch, im Plan und im ausgegebenen PDF. Öffne das Projekt danach neu. | neuer Name in den Cues des Drehbuchs, im Plan und im PDF, nach Neustart | D-08, LH-03, LH-80 |
| K-02 | P1 | Versetz eine Figur an einem Mark, leg dafür einen neuen Shot an und gib danach Plan und Shotliste aus. | Mark versetzt, Shot da, beide Ausgaben zeigen es | LH-08, LH-70, LH-80 |
| K-03 | P3 | Leg einen neuen Shot für Kamera A an, ändere seine Beschreibung und prüfe, ob er überall, wo das Programm Shots zeigt, gleich erscheint. | Shot in Kacheln, Tabelle und Zeitleiste mit gleichen Werten | LH-70, LH-61 |
| K-04 | P2 | Change the lens of a camera during a shot (a zoom) and check that the frame, the shot list and the printed shot list all agree. | Zoom zwischen zwei Marks, Bewegung „Zoom“ vorgeschlagen oder gesetzt, PDF-Shotliste gleich | LH-33, LH-36, LH-80 |

## Frei

| Id | Persona | Aufgabe |
|---|---|---|
| F-01 | P1 | Zehn Minuten: Was würdest du mit dieser Szene als Nächstes machen? Versuch es. |
| F-02 | P4 | Zehn Minuten: Schau dich um, was dir das Programm für deine Arbeit bietet. |
| F-03 | P5 | Zehn Minuten: Wie würdest du hier ein Set für einen neuen Drehort anlegen? Versuch es. |

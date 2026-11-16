# Klappe: Aufgaben für QA-Runden

Aufgaben für die Personas aus [`PERSONAS.md`](PERSONAS.md), Verfahren im Skill `qa-personas`.
Jede Aufgabe nennt ein Ziel in den Worten der Persona, nie den Weg und keinen Namen eines
Bedienelements. Der Tester bekommt nur den Text unter „Aufgabe“; „Endzustand“ und „Bezug“
prüft der Overseer. Ausgang je Aufgabe: erreicht, mit Umweg, nicht gefunden, nicht prüfbar.

Start: `scripts/qa-session.sh start <arbeitsordner>` öffnet die Startseite; das Beispielprojekt (Szene 14 aus
„Nachtwache“, de oder en) liegt dort als Kachel. Mit `--open` öffnet die Sitzung es direkt.
Aufgaben mit Dateidialog sind „nicht prüfbar“, bis der Test des Sitzungsskripts zeigt, dass der
Dialog in der Sitzung erscheint; Ausgaben prüft der Overseer dann über `klappe-cli`.

Ausgangslage im Beispielprojekt (für den Overseer): Ilva sitzt am Tisch, hat einen Mark und geht
bei 34 s ab; der Wärter ist von 3 bis 40 s da und hat einen Knick auf dem Weg zu Mark 2 (8 s);
Kameras A (Marks bei 0 und 29 s) und B (Super 35, 50 mm, auf Ilva); fünf Shots ohne
Achsensprung; Achse Ilva–Wärter (nur 3 bis 34 s, solange beide da sind); drei Lampen; Set mit
Tür (rechts), zwei Fenstern (oben, links), Tisch (0,75 m hoch), Stuhl, Wendeltreppe und der
Notiz „Nebelmaschine außen“. Ein Endzustand nennt nur, was sich
gegenüber dieser Lage ändert.

Eine Runde nimmt höchstens drei Personas mit je vier bis sechs Aufgaben. Neue Aufgaben kommen
dazu, wenn `STAND.md` einen Punkt als fertig oder in erster Fassung führt. Nicht abgedeckt: die
TUI (eigene Runde, wenn gewünscht), das Hintergrundbild (LH-13; das Beispiel hat kein Bild) und
die Modi (D-01; jede Aufgabe benutzt sie nebenbei, eine eigene Aufgabe müsste ihre Namen nennen).

## Abdeckung

| Id | Persona | Aufgabe | Endzustand | Bezug |
|---|---|---|---|---|
| A-01 | P1 | Öffne das Beispielprojekt und schau dir die Szene an, als würdest du sie zum ersten Mal vorbereiten. Lass die Szene einmal von Anfang bis Ende ablaufen und schau dabei, was Kamera A sieht. | Projekt offen, Szene bis zum Ende gespielt, Kamerablick folgt der Zeit | LH-60, LH-62, LH-38, D-06, D-07 |
| A-02 | P1 | Stell eine zusätzliche Figur in die Szene, die durch die Tür hereingerannt kommt, im Bogen (nicht gerade) zum Tisch geht, kurz wartet und sich setzt. | neue Figur; erscheint an oder vor der Tür (Auftritt oder Weg von außen); erster Weg im Renntempo; Weg zum Tisch mit mindestens einem Knick; Pause am Tisch, letzter Mark „sitzend“ | LH-03, LH-21, LH-22, LH-26, LH-63 |
| A-03 | P1 | Der Wärter soll auf dem Weg zu seinem zweiten Platz geradeaus gehen, ohne Bogen. | Weg des Wärters zu Mark 2 ohne Knick, Mark 2 unverändert, kein neuer Mark | LH-21 |
| A-04 | P1 | Probier eine zweite Fassung der Auflösung, ohne die erste zu verlieren, und geh danach zur ersten zurück. | zwei Pläne, der erste gezeigt, unverändert | LH-04 |
| A-05 | P2 | Add a third camera for a close-up of the keeper: full-frame body, 85 mm lens. Check what it sees. | Kamera C mit Full Frame, 85 mm, Blick auf den Wärter; Kamerablick zeigt ihn (zwischen 3 und 40 s) | LH-32, LH-36 |
| A-06 | P2 | When the keeper walks to his second mark, camera A should dolly alongside him with the lens at 1.2 m height, tilted 10° down, on a track laid on the floor. | Kamera-Marks von A zwischen 3 und 8 s, Gerät Dolly oder Schiene, Höhe 1,2 m, Neigung −10°; ein Schienenobjekt unter dem Weg von A | LH-33, LH-34, LH-35 |
| A-07 | P2 | Find out whether any shot breaks the 180° line. Then add a shot from the other side of the line and check again. | erst: keiner; dann neuer Shot (Beginn zwischen 3 und 34 s), den das Programm als Achsensprung zeigt | LH-40, LH-41, LH-70 |
| A-08 | P3 | Bring die Shots in die Reihenfolge, in der am Drehtag gedreht wird, und prüfe, welche Shots dieselbe Kameraposition haben. | Drehreihenfolge gezeigt, Setups erkennbar | LH-71, LH-72 |
| A-09 | P3 | Gib die Shotliste so aus, dass die Produktion sie in ihrer Tabellenkalkulation öffnen kann. | ODS oder XLSX; CSV ist ein Umweg | LH-85 |
| A-10 | P3 | Ändere bei einem Shot die Einstellungsgröße und die Beschreibung und stell sicher, dass es nach dem Schließen und Wiederöffnen noch so ist. | Werte nach Neustart erhalten | LH-05, LH-70, LH-71 |
| A-11 | P4 | Stell zwei weitere Lampen in den Plan: eine Stufenlinse mit 2 kW durchs Fenster und ein LED-Panel mit 3200 K auf Ilva. Verschieb dabei nichts anderes. | zwei neue Lampen mit Werten; Figuren, Kameras, Set unverändert | LH-50, LH-51, LH-15 |
| A-12 | P4 | Druck einen Plan, auf dem deine Lampen mit ihren Werten stehen. | PDF mit Lampenliste | LH-80 |
| A-13 | P2 | Later the keeper's second mark moves half a metre. Camera A should move with him without redoing its marks. | A folgt dem Wärter; nach Verschieben von Mark 2 ist der Mark von A bei 29 s mitgewandert (Fenster 8 bis 36 s), der bei 0 s nicht | LH-37 |
| A-14 | P2 | Set up one new shot completely without touching the mouse. | neuer Shot mit Kamera, Größe, Objektiv; `actions.log` ohne Klick und Ziehen | NF-06, D-02 |
| A-15 | P5 | Der Raum bekommt eine Nische: eine zusätzliche Wand mit einer Tür, davor ein Sofa und ein Regal (2 × 0,4 m), das du auch in späteren Szenen brauchst. Der Tisch ist in Wirklichkeit 90 cm hoch. Miss, wie breit der Durchgang zwischen Tisch und Wendeltreppe ist. | neue Wand mit Tür, Sofa; eigenes Objekt „Regal“ 2 × 0,4 m in der Bibliothek des Projekts und im Plan; Tisch 0,9 m hoch; Messwert etwa 1,5 m (Tischecke bis Treppenrand) | LH-10, LH-11, LH-12, LH-14, LH-17, LH-25 |
| A-16 | P5 | Zeichne mit einem Pfeil ein, wo die Requisite den Weg frei halten muss, schreib eine Notiz für das Team dazu und gib den Plan als Bild für eine Präsentation aus. | neuer Pfeil und neue Notiz im Plan, beide im PNG | LH-16, LH-19, LH-81 |
| A-17 | P1 | An der Tür sollen zwölf Gäste stehen, ohne zwölf Figuren einzeln hinzustellen. | eine Gruppe mit Zahl 12 an der Tür | LH-23 |
| A-18 | P1 | Setz für ein neues Gespräch eine übliche Schuss-Gegenschuss-Auflösung ein, ohne sie von Hand zu bauen. Mach danach drei Änderungen und nimm zwei davon zurück. | Vorlage eingefügt; eine Änderung bleibt | LH-07, LH-06 |
| A-19 | P4 | Die Stufenlinse am Fenster leuchtet zu breit: mach ihren Lichtkegel enger und richte sie genau auf den Tisch. | Abstrahlwinkel der Lampe kleiner, Richtung auf den Tisch; Ort der Lampe unverändert | LH-50 |
| A-20 | P4 | Die Produktion möchte deine Lampen in ihrer Tabellenkalkulation haben. | ODS oder XLSX mit einem Blatt der Lampen (Name, Art, W, K, Winkel) | LH-50, LH-85 |
| A-21 | P5 | Die Tür rechts ist in Wirklichkeit 1,10 m breit, und das obere Fenster ist 20 cm schmaler als im Plan. | Tür 1,10 m, oberes Fenster 0,20 m schmaler; beide bleiben in ihrer Wand | LH-10 |
| A-22 | P5 | Der Tisch wird 2,40 m lang und 1 m tief gebaut. Stell ihn so in den Plan. | Tisch 2,40 × 1,00 m, an seinem Platz | LH-25 |
| A-23 | P5 | Zeichne ein Podest mit einer abgeschrägten Ecke ein. Danach ändert sich der Bau: eine Ecke liegt einen halben Meter weiter außen. | freie Form mit fünf Punkten; ein Punkt danach 0,5 m nach außen versetzt, die anderen unverändert | LH-92 |
| A-24 | P5 | Die Notiz „Nebelmaschine außen“ ist breiter als ihr Text und ragt in den Raum. Mach sie so schmal, dass der Text gerade noch auf zwei Zeilen passt. | Notiz schmaler (unter 2,4 m), Text auf zwei Zeilen | LH-16 |
| A-25 | P1 | Du hast zwei Fassungen der Auflösung. Die zweite gefällt dir besser: sie soll in der Liste als erste stehen. | nach A-04: zweiter Plan steht vorn, beide unverändert | LH-04 |
| A-26 | P1 | Schick dem Team den Plan auf einem hochkant liegenden A4-Blatt. Beim nächsten Mal soll das Programm das schon so vorschlagen. | PDF hochkant A4; Projekt speichert `a4-portrait`, der Dialog schlägt es vor | LH-80 |

## Ketten

| Id | Persona | Aufgabe | Endzustand | Bezug |
|---|---|---|---|---|
| K-01 | P1 | Der Wärter heißt jetzt anders. Benenne ihn um und prüfe, ob der neue Name überall steht: im Drehbuch, im Plan und im ausgegebenen PDF. Öffne das Projekt danach neu. | neuer Name in den Cues des Drehbuchs, im Plan und im PDF (ausgegeben zwischen 3 und 40 s, wenn der Wärter da ist), nach Neustart | D-08, LH-03, LH-80 |
| K-02 | P1 | Versetz eine Figur an einem Mark, leg dafür einen neuen Shot an und gib danach Plan und Shotliste aus. | Mark versetzt, Shot da, beide Ausgaben zeigen es | LH-08, LH-70, LH-80 |
| K-03 | P3 | Leg einen neuen Shot für Kamera A an, ändere seine Beschreibung und prüfe, ob er überall, wo das Programm Shots zeigt, gleich erscheint. | Shot in Kacheln, Tabelle und Zeitleiste mit gleichen Werten | LH-70, LH-61 |
| K-04 | P2 | Change the lens of a camera during a shot (a zoom) and check that the frame, the shot list and the printed shot list all agree. | Zoom zwischen zwei Marks, Bewegung „Zoom“ vorgeschlagen oder gesetzt, PDF-Shotliste gleich | LH-33, LH-36, LH-80 |

## Frei

| Id | Persona | Aufgabe |
|---|---|---|
| F-01 | P1 | Zehn Minuten: Was würdest du mit dieser Szene als Nächstes machen? Versuch es. |
| F-02 | P4 | Zehn Minuten: Schau dich um, was dir das Programm für deine Arbeit bietet. |
| F-03 | P5 | Zehn Minuten: Wie würdest du hier ein Set für einen neuen Drehort anlegen? Versuch es. |

# Klappe: Aufgaben für QA-Runden

Aufgaben für die Personas aus [`PERSONAS.md`](PERSONAS.md), Verfahren im Skill `qa-personas`.
Jede Aufgabe nennt ein Ziel in den Worten der Persona, nie den Weg und keinen Namen eines
Bedienelements. Der Tester bekommt nur den Text unter „Aufgabe“; „Endzustand“ und „Bezug“
prüft der Overseer. Ausgang je Aufgabe: erreicht, mit Umweg, nicht gefunden, nicht prüfbar.

Start: `scripts/qa-session.sh` mit dem Beispielprojekt (Szene 14 aus „Nachtwache“, de oder en).
Aufgaben mit Dateidialog sind „nicht prüfbar“, bis der Test des Sitzungsskripts zeigt, dass der
Dialog in der Sitzung erscheint; Ausgaben prüft der Overseer dann über `klappe-cli`.

Eine Runde nimmt höchstens drei Personas mit je vier bis sechs Aufgaben. Neue Aufgaben kommen
dazu, wenn `STAND.md` einen Punkt als fertig oder in erster Fassung führt.

## Abdeckung

| Id | Persona | Aufgabe | Endzustand | Bezug |
|---|---|---|---|---|
| A-01 | P1 | Öffne das Beispielprojekt und schau dir die Szene an, als würdest du sie zum ersten Mal vorbereiten. Lass die Szene einmal von Anfang bis Ende ablaufen. | Szene gespielt, Zeit am Ende | LH-60, D-06, D-07 |
| A-02 | P1 | Stell eine zusätzliche Figur in die Szene, die durch die Tür hereinkommt, zum Tisch geht und sich setzt. | neue Figur mit Auftritt, Weg, Mark „sitzend“ | LH-03, LH-21, LH-22, LH-26 |
| A-03 | P1 | Der Weg einer Figur soll nicht gerade, sondern im Bogen um den Tisch führen. | Weg mit mindestens einem Knick | LH-21 |
| A-04 | P1 | Probier eine zweite Fassung der Auflösung, ohne die erste zu verlieren, und geh danach zur ersten zurück. | zwei Pläne, der erste gezeigt, unverändert | LH-04 |
| A-05 | P2 | Set up a second camera for a reverse shot on the other character, with a 50 mm lens on a Super 35 body, and check what that camera sees. | Kamera mit Körper und Objektiv, Kamerablick sichtbar | LH-32, LH-36 |
| A-06 | P2 | Make one camera dolly along with a walking character, and give the move a fixed height and tilt. | Kamera-Marks mit Gerät, Höhe, Neigung; „folgt“ gesetzt | LH-33, LH-34, LH-37 |
| A-07 | P2 | Find out whether any of the shots break the 180° line. | Achse gesetzt, Achsensprung gefunden und benannt | LH-40, LH-41 |
| A-08 | P3 | Bring die Shots in die Reihenfolge, in der am Drehtag gedreht wird, und prüfe, welche Shots dieselbe Kameraposition haben. | Drehreihenfolge gezeigt, Setups erkennbar | LH-71, LH-72, LH-77 |
| A-09 | P3 | Gib die Shotliste so aus, dass die Produktion sie in ihrer Tabellenkalkulation öffnen kann. | ODS oder XLSX im Ausgabeordner | LH-85 |
| A-10 | P3 | Ändere bei einem Shot die Einstellungsgröße und die Beschreibung und stell sicher, dass es nach dem Schließen und Wiederöffnen noch so ist. | Werte nach Neustart erhalten | LH-05, LH-70 |
| A-11 | P4 | Stell zwei weitere Lampen in den Plan: eine Stufenlinse mit 2 kW durchs Fenster und ein LED-Panel mit 3200 K auf die sitzende Figur. Verschieb dabei nichts anderes. | zwei Lampen mit Werten; Figuren, Kameras, Set unverändert | LH-50, LH-51, LH-15 |
| A-12 | P4 | Druck einen Plan, auf dem deine Lampen mit ihren Werten stehen. | PDF mit Lampenliste | LH-80 |

## Ketten

| Id | Persona | Aufgabe | Endzustand | Bezug |
|---|---|---|---|---|
| K-01 | P1 | Eine Figur heißt jetzt anders. Benenne sie um und prüfe, ob der neue Name überall steht: im Drehbuch, im Plan, in der Shotliste und im ausgegebenen PDF. Öffne das Projekt danach neu. | neuer Name in Drehbuch-Cues, Plan, Shotliste, PDF, nach Neustart | D-08, LH-03, LH-80 |
| K-02 | P1 | Versetz eine Figur an einem Mark, leg dafür einen neuen Shot an und gib danach Plan und Shotliste aus. | Mark versetzt, Shot da, beide Ausgaben zeigen es | LH-08, LH-70, LH-80 |
| K-03 | P3 | Leg einen neuen Shot im Plan an, bearbeite ihn in der Tabelle und finde ihn danach im Plan und in der Zeitleiste wieder. | Shot in Plan, Tabelle, Zeitleiste gleich | LH-70, LH-61 |
| K-04 | P2 | Change the lens of a camera mid-shot (a zoom) and check that the frame, the shot list and the export all agree. | Zoom zwischen zwei Marks, Bewegung „Zoom“, Export | LH-33, LH-36, LH-85 |

## Frei

| Id | Persona | Aufgabe |
|---|---|---|
| F-01 | P1 | Zehn Minuten: Was würdest du mit dieser Szene als Nächstes machen? Versuch es. |
| F-02 | P4 | Zehn Minuten: Schau dich um, was dir das Programm für deine Arbeit bietet. |

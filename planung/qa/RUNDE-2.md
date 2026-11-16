# QA-Runde 2, Klappe, 16.11.2026, Stand 18eb4a1

Drei Testpersonen (P1 Jana, Regie; P4 Ole, Licht; P5 Sanne, Ausstattung) haben 13 Aufgaben
gespielt, vor allem die neuen Griffe, Pläne, Papier und Lampen. Jede spielte in einer eigenen,
abgeschotteten Sitzung mit dem Beispielprojekt. Verfahren: Skill `qa-personas`, Aufgaben:
[`../QA.md`](../QA.md) (neu: A-19 bis A-26).

## Kurz

Erreicht haben die Personas Pläne anlegen, wechseln und umsortieren, Druck im Hochformat mit
gemerktem Papier, Tür, Fenster und Tisch nach Maß, eine schmalere Notiz und eine Lampe mit
engerem Kegel. Nicht erreicht haben sie eine Tabelle der Lampen, weil die Tabellen-Ausgabe
weiter keinen Dialog öffnet (Q1-01), und eine Farbtemperatur am LED-Panel, weil die Karte den
alten Wert zeigt (Q2-02). Am meisten stören die Felder ohne Beschriftung: Ole musste die vier
Zahlen der Lampe durch Ausprobieren deuten, Sanne raten, welches Feld des Tischs die Länge ist.

## Die wichtigsten Funde

1. Q1-01 (aus Runde 1, wieder gesehen) Die Tabellen-Ausgabe (CSV, ODS, XLSX) öffnet keinen
   Dialog; damit ist auch die Lampenliste unerreichbar. Behebung vorbereitet: #158.
2. Q2-02 Eine neue Farbtemperatur oder Leistung auf der Lampenkarte wird gesetzt, das Feld
   zeigt aber wieder den alten Wert. Behebung vorbereitet: #157.
3. Q2-03 Die Zahlenfelder der Karten von Lampe, Tür und Möbel haben keine Beschriftung
   (Fortsetzung von Q1-03).
4. Q2-01 Das Planmenü zeigt jeden Befehl zweimal. Behebung vorbereitet: #156.
5. Q2-04 Strg+D an einer Figur meldet nur „Ungültiger Wert für Element:“.

## Aufgaben

| Aufgabe | Persona | Ausgang |
|---|---|---|
| A-04 zweite Fassung, erste behalten | P1 | erreicht (neuer Plan ist leer; „Schnappschuss…“ für eine Kopie nicht probiert) |
| A-25 Pläne umsortieren | P1 | erreicht |
| A-26 A4 hoch, gemerkt | P1 | erreicht (nur innerhalb der Sitzung geprüft) |
| A-17 zwölf Gäste | P1 | mit Umweg (Gruppe nur über Datei › Projekt › Figuren) |
| F-01 frei | P1 | Kamera, Shot, Abspielen, Speichern erreicht |
| A-11 zwei Lampen stellen | P4 | mit Umweg; 3200 K am LED-Panel nicht erreicht (Q2-02) |
| A-19 Kegel enger, auf den Tisch | P4 | mit Umweg (Felder durch Probieren gedeutet; Griff lag unter der Karte, Q1-04) |
| A-12 Plan mit Lampenwerten drucken | P4 | erreicht über PDF |
| A-20 Lampen als Tabelle | P4 | nicht gefunden; vorhanden (Blatt „Lampen“ in ODS/XLSX), aber der Dialog kommt nicht (Q1-01) |
| A-21 Tür 1,10 m, Fenster 20 cm schmaler | P5 | erreicht |
| A-22 Tisch 2,40 × 1 m | P5 | erreicht (Felder geraten, Q2-03) |
| A-23 freie Form, Ecke versetzen | P5 | Form erreicht; Ecke nicht prüfbar: der Stand der Runde ist vor #154 (Punkte ziehen) |
| A-24 Notiz schmaler, zwei Zeilen | P5 | erreicht |

## Funde

### Q2-01 Das Planmenü zeigt jeden Befehl zweimal
- **Art:** Fehler · **Stufe:** mittel
- **Wo:** P1, A-04 und A-25
- **Getan / Gesehen / Erwartet:** Plan-Menü geöffnet. Unter den gestalteten Einträgen steht
  eine zweite, ungestaltete Liste derselben Befehle. Erwartet: jeder Befehl einmal (LH-04).
- **Beleg:** P1 `shot-004`, `shot-008`, `shot-010`, `shot-011`
- **Nachgeprüft:** ja, frische Sitzung (`check/shot-002`)
- **Einordnung:** In Runde 1 als „Plan-Menü doppelt gezeichnet“ nicht nachgespielt; es lag
  nicht an der Sitzung. Die Befehle standen im Inhalt des Menüs, und das Menü macht aus jedem
  einen zweiten Eintrag.
- **Status:** offen (Behebung vorbereitet: #156)

### Q2-02 Farbtemperatur und Leistung springen auf der Lampenkarte zurück
- **Art:** Fehler · **Stufe:** hoch
- **Wo:** P4, A-11
- **Getan / Gesehen / Erwartet:** Am LED-Panel 3200 (später 4000) ins vierte Feld getippt,
  „Übernehmen“ oder Enter. Das Feld zeigt wieder 5600, keine Meldung. Erwartet: 3200 K (LH-50).
- **Beleg:** P4 `shot-026` bis `shot-034`, `actions.log` 05:24:51 bis 05:25:11
- **Nachgeprüft:** ja, frische Sitzung: Schreibtischlampe 3200 → 4000, das Feld zeigt 3200
  (`check/shot-005`, `shot-006`); gespeichert steht 4000 K in der Szene.
- **Einordnung:** Der Wert wird gesetzt, die Karte zeigt ihn nur nicht, weil sie die Lampe
  nur neu liest, wenn sich das Bild des Plans ändert. Farbe und Leistung zeichnet der Plan
  nicht. Ole hielt den Wert deshalb für nicht übernommen.
- **Status:** offen (Behebung vorbereitet: #157)

### Q2-03 Die Zahlenfelder von Lampe, Tür und Möbel haben keine Beschriftung
- **Art:** unverständlich · **Stufe:** mittel
- **Wo:** P4 A-11, A-19; P5 A-21, A-22
- **Getan / Gesehen / Erwartet:** Lampe: vier Felder (0 / 30 / 2000 / 3200), Ole fand nur durch
  Ändern heraus, was Richtung, Kegel, Leistung und Kelvin sind; kein Tooltip. Tür: Breitenfeld
  ohne Namen. Tisch: zwei Felder, Sanne musste raten, welches die Länge ist. Keines nennt die
  Einheit. Erwartet: „Breite (m)“, „Länge“, „Tiefe“, „Kegel (°)“ usw.
- **Beleg:** P4 `shot-010`, `shot-011`, `shot-014`; P5 `shot-004`, `shot-009`
- **Nachgeprüft:** ja (`check/shot-003`, Lampenkarte ohne Beschriftung)
- **Einordnung:** Fortsetzung von Q1-03 (Objekt, Mark, Kamera); die Namen stehen nur als
  Platzhalter im leeren Feld.
- **Status:** offen

### Q2-04 Strg+D an einer Figur meldet nur „Ungültiger Wert für Element:“
- **Art:** unverständlich · **Stufe:** niedrig
- **Wo:** P1, F-01
- **Getan / Gesehen / Erwartet:** Figur gewählt, Strg+D, um sie zu kopieren. Oben steht
  „Ungültiger Wert für Element:“ ohne Wert, dazu öffnet sich die Karte des Marks. Erwartet:
  eine Kopie oder eine verständliche Absage.
- **Beleg:** P1 `shot-024` bis `shot-026`, `actions.log` 05:23:59
- **Nachgeprüft:** ja (`check/shot-015`)
- **Einordnung:** Das Kopieren einer Figur ist nicht vorgesehen; die Absage nennt das nicht.
- **Status:** offen

### Q1-01 (Runde 1) Die Tabellen-Ausgabe öffnet keinen Dialog
- **Wo:** P4, A-20 (Shotliste ODS, dreimal)
- **Beleg:** P4 `shot-034` bis `shot-038`
- **Nachgeprüft:** ja, ODS und CSV aus dem Menü und ODS aus der Befehlspalette
  (`check/shot-010`, `shot-012`, `shot-014`)
- **Einordnung:** Der Dateidialog hatte kein Fenster, solange die Papierfrage nicht offen war;
  bei PDF ist sie offen, darum ging es dort. Mit der Behebung erscheint der Dialog, und die
  ODS-Datei hat die Blätter „Shotliste“ und „Lampen“ (A-20).
- **Status:** offen (Behebung vorbereitet: #158)

## Nicht bestätigt

Nicht nachgespielt; die Bilder liegen unter `~/.cache/claude/klappe-qa/runde-2/`.

- **Gruppe von zwölf zeigt drei Punkte; Figuren nur über Datei › Projekt** (P1 `shot-031` bis
  `shot-038`): die Gruppe ist vorhanden, aber nicht dort, wo die Persona sie suchte.
- **Kürzel füllt sich beim Tippen des Namens selbst** („G“, getippt „GG“; P1 `shot-034`).
- **Statusmeldung abgeschnitten und bleibt nach dem Planwechsel stehen** („Der Plan hat keine
  Kamera: le…“; P1 `shot-007`, `shot-009`); in Runde 1 schon gesehen.
- **Ausgabedialog nennt nicht, welcher Plan ausgegeben wird** (P1 `shot-013`).
- **Shot-Bereich unten abgeschnitten** (P1 `shot-040`; Mausrad kam erst nach dem Ende).
- **Lampen nur unter „Auflösen“** (P4 `shot-004` bis `shot-007`): Ole suchte sie unter „Set
  bauen“.
- **Lampe L1 am oberen Rand des PDF abgeschnitten** (P4 `pdfseite-1.png`).
- **Keine Lampe „auf etwas richten“** (P4 A-19): Richtung nur als Zahl oder per Griff.
- **Notizbreite nicht als Zahl** (P5 A-24); **Tür zeigt „1,1“ statt „1,10“** (P5 `shot-025`).
- **Blasser Rest „HÖHE I…“ unter der Karte** (P5 `shot-017`, `shot-018`).
- **Kein Ladehinweis nach Klick auf die Beispiel-Kachel** (P1, P4 `shot-002`); wie in Runde 1.

Kein Fund: Der Griff der Lampe „bewirkte nichts“ (P4 `shot-017`), weil er unter der Lampenkarte
lag und der Zug die Karte traf (Q1-04). Die Tischkarte beim Zeichnen der Form (P5 `shot-012`)
gehört zum noch gewählten Tisch.

## Nicht getestet

- A-23, zweite Hälfte (Punkte ziehen): kam mit #154 nach dem Stand der Runde.
- P1: Neustart (gemerktes Papier), Inhalt der PDF, Schnappschuss, Vorlagen, Rückgängig.
- P4: Inhalt der Tabellen (Dialog kam nicht), PNG, SVG, Speichern.
- P5: Ausgabe, 3D, Messen, Neuöffnen, Rechtsklick (das Werkzeug hat keinen).

## Vorfälle (Testumgebung)

- Keine: kein Fenster, kein Dialog, keine Datei außerhalb der Sitzungen; alle Dateidialoge
  zeigten nur den Ordner der Sitzung.
- **Werkzeug:** Der Dateidialog in der Sitzung ist der englische Ersatzdialog von Qt (kein
  Plasma in der Sitzung); Enter bestätigt dort nicht (P4). Das Mausrad (`click --button 4/5`)
  kam für P1 und P4 erst als Nachtrag; es gehört ins Briefing.

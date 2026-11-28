# QA-Runde 1, Klappe, 15.11.2026, Stand 19cccf4

Vier Testpersonen (Probelauf P1 Jana, Regie; Runde mit P2 Mehmet, Kamera, Englisch; P3 Katrin,
Regieassistenz am Laptop; P5 Sanne, Ausstattung) haben 16 Aufgaben am laufenden Programm
gespielt, jede in einer eigenen, abgeschotteten Sitzung mit dem Beispielprojekt. Verfahren:
Skill `qa-personas`, Aufgaben: [`../QA.md`](../QA.md).

## Kurz

Die Personas kamen in den Plan, konnten abspielen, Objekte, Wände, Figuren und Kameras setzen,
Werte ändern und PDF und Bild ausgeben. Nicht erreicht haben sie die Shotliste als Tabelle (der
Menüpunkt tut nichts), das Schließen eines Projekts und die 180°-Prüfung; viele Felder sind ohne
Beschriftung nicht zu deuten. Am meisten stören die Karten am Element: ohne Beschriftung, und
sie bleiben offen und verdecken den Plan.

## Die wichtigsten Funde

1. Q1-01 „Shotliste CSV“ tut nichts: kein Dialog, keine Datei, keine Meldung.
2. Q1-02 Ein Projekt lässt sich nicht schließen; es gibt keinen Weg zurück zur Startseite.
3. Q1-03 Die Zahlenfelder in den Karten (Objekt, Mark, Kamera) haben keine Beschriftung.
4. Q1-04 Die Karte am Element schließt weder mit Escape noch nach „Übernehmen“ noch beim
   Werkzeug- oder Moduswechsel und verdeckt den Plan.
5. Q1-05 Mit dem Figur-Werkzeug lässt sich keine neue Figur anlegen.
6. Q1-06 Eine Ausgabe ohne getippte Endung wird ohne Endung gespeichert („plan“ statt
   „plan.pdf“).
7. Q1-07 Plan-Ausgabe schneidet ab, was außerhalb des Raums liegt (Notiz fehlt, Sofa halb), und
   kürzt Notizen („Nebelmaschine a…“).
8. Q1-08 Die 180°-Prüfung ist nicht auffindbar.
9. Q1-09 Der Kamerablick erscheint nur bei gewähltem Shot und folgt beim Abspielen nicht dem
   Schnitt.
10. Q1-10 Im Shot-Bereich liegen die Felder eines Shots nur durch Scrollen erreichbar unter den
    Kacheln, die beim Scrollen darüber liegen bleiben.

## Aufgaben

| Aufgabe | Persona | Ausgang |
|---|---|---|
| A-01 Szene ansehen, abspielen, Kamera A | P1 | mit Umweg (Kamerablick erst nach Wahl eines Shots) |
| A-02 Neue Figur durch die Tür, Bogen, sitzen | P1 | nicht gefunden (Bogen); Figur mit Umweg über Datei › Projekt |
| K-01 Wärter umbenennen, überall prüfen | P1 | nicht prüfbar (PDF, Neuöffnen); Drehbuch und Plan erreicht |
| A-05 Dritte Kamera, Full Frame, 85 mm | P2 | mit Umweg (Kamera nur im Modus „Auflösen“); Blick auf den Wärter nicht erreicht |
| A-06 Fahrt neben dem Wärter, Höhe, Neigung, Schiene | P2 | nicht gefunden (Schiene am Boden); Höhe, Neigung, Gerät erreicht |
| A-07 Achsensprung finden, Gegenschuss | P2 | nicht gefunden |
| A-13 Kamera folgt dem Wärter | P2 | nicht gefunden (Mark um genau 0,5 m); „Folgt“ erreicht |
| A-14 Shot nur mit der Tastatur | P2 | mit Umweg (Tab bis „Neuer Shot“) |
| K-04 Zoom im Shot, Bild, Liste, Druck | P2 | nicht prüfbar (keine Shotliste ausgebbar) |
| A-08 Drehreihenfolge, gleiche Kameraposition | P3 | mit Umweg |
| A-09 Shotliste für die Tabellenkalkulation | P3 | nicht gefunden (Menüpunkt tut nichts) |
| A-10 Größe und Inhalt ändern, neu öffnen | P3 | nicht gefunden (Inhalt); nicht prüfbar (Neuöffnen); Größe erreicht |
| K-03 Neuer Shot überall gleich | P3 | mit Umweg; Abweichungen siehe „Nicht bestätigt“ |
| A-15 Nische, Sofa, Regal, Tischhöhe, Messen | P5 | mit Umweg |
| A-16 Pfeil, Notiz, als Bild | P5 | mit Umweg (Notiz fehlt im Bild) |
| F-03 Set für neuen Drehort | P5 | mit Umweg („Neuer Plan“ übernimmt den Raum) |

## Funde

„Nachgeprüft: ja“ heißt: in frischer Sitzung nachgespielt (`check`, `check2`, `check3`). Die übrigen
sind an Stand, Code oder den Dateien der Testsitzung abgeglichen und vor einer Behebung
nachzuspielen.

### Q1-01 „Shotliste CSV“ tut nichts
- **Art:** Fehler · **Stufe:** hoch
- **Wo:** P2 Aufgabe 6, P3 Aufgabe 2 (A-09, K-04)
- **Getan / Gesehen / Erwartet:** Ausgeben › Shotliste CSV, mehrfach. Kein Dialog, keine
  Meldung, keine Datei. Erwartet: Speichern-Dialog wie bei PDF (LH-85, LH-80).
- **Beleg:** P3 `shot-011`, `shot-013`, `shot-017`; P2 `shot-057`
- **Nachgeprüft:** ja, frische Sitzung (`check/shot-003`): kein Dialog, keine Datei, kein
  Eintrag im Protokoll.
- **Einordnung:** Der Menüpunkt ruft den Dateidialog ohne Papierfrage auf; der kommt nicht.
  ODS und XLSX fehlen in der App ohnehin (STAND).
- **Status:** offen

### Q1-02 Ein Projekt lässt sich nicht schließen
- **Art:** fehlt in der Oberfläche · **Stufe:** hoch
- **Wo:** P1 K-01, P3 A-10
- **Getan / Gesehen / Erwartet:** Datei-Menü zeigt Neu…, Öffnen…, Projekt…, Speichern,
  Hintergrund…; kein Schließen. Klick auf „KLAPPE“, Titel, Strg+W: nichts. „Öffnen…“ startet
  in einem leeren Ordner, ohne zuletzt geöffnete Projekte. Erwartet: zurück zur Startseite mit
  den Projektkacheln (D-07, LH-01).
- **Beleg:** P1 `shot-046`; P3 `shot-014`, `shot-029` bis `shot-031`
- **Nachgeprüft:** nein; abgeglichen am Menü und im Code (kein Eintrag zum Schließen).
- **Einordnung:** fehlt in der Oberfläche
- **Status:** offen (Behebung vorbereitet: #168)

### Q1-03 Felder in den Karten ohne Beschriftung
- **Art:** unverständlich · **Stufe:** hoch
- **Wo:** P1 A-02 (Mark), P2 A-05/A-06 (Kamera, Mark), P5 A-15 (Objekt)
- **Getan / Gesehen / Erwartet:** Die Karten zeigen Zahlen ohne Namen und Einheit: Objekt
  „1,4“ „0,8“ „0,75“, Mark „0“ bzw. „19“ „1,5“, Kamera „24,89“ „2,39“. Was Breite, Tiefe,
  Höhe, Pause, Höhe oder Seitenverhältnis ist, mussten alle raten; nur Tooltips helfen.
  Erwartet: Name und Einheit an jedem Feld (D-04).
- **Beleg:** P5 `shot-013`; P2 `shot-025`, `shot-038`; P1 `shot-030`
- **Nachgeprüft:** ja (`check/shot-004`, `check/shot-007`)
- **Status:** offen (Behebung vorbereitet: #165)

### Q1-04 Karte am Element bleibt offen und verdeckt den Plan
- **Art:** Reibung · **Stufe:** mittel
- **Wo:** P2 A-05, P5 A-15/A-16
- **Getan / Gesehen / Erwartet:** Nach „Übernehmen“, Escape, Wechsel von Werkzeug, Modus oder
  Auswahl bleibt die Karte stehen, teils mit dem vorigen Element. Erwartet: sie schließt (D-04).
- **Beleg:** P5 `shot-022` bis `shot-035`; P2 `shot-033`
- **Nachgeprüft:** ja (`check/shot-005`, `-006`, `-013`: Karte „Tisch“ bleibt nach Escape,
  Werkzeug- und Moduswechsel)
- **Status:** offen (Behebung vorbereitet: #173)

### Q1-05 Neue Figur nicht mit dem Figur-Werkzeug
- **Art:** unverständlich · **Stufe:** mittel
- **Wo:** P1 A-02
- **Getan / Gesehen / Erwartet:** Figur-Werkzeug, Klick in den Plan: setzt einen Mark für Ilva.
  Die Liste neben dem Werkzeug kennt nur „Ilva Brandes“ und „Wärter“. Eine neue Figur gab es
  erst über Datei › Projekt… › Figuren › Neu. Bei der Kamera gibt es „Neue Kamera“ in der
  Liste (STAND). Erwartet: dasselbe für Figuren (LH-03).
- **Beleg:** P1 (Probelauf) `shot-014`, `shot-015`
- **Nachgeprüft:** ja (`check/shot-015`)
- **Status:** offen

### Q1-06 Ausgabe ohne Endung gespeichert
- **Art:** Fehler · **Stufe:** mittel
- **Wo:** P1 K-01; P5 A-16
- **Getan / Gesehen / Erwartet:** PDF als „plan“ gespeichert: die Datei heißt `plan`, ohne
  `.pdf`. Return im Dateidialog speicherte nicht, erst der Knopf. Erwartet: Endung wird ergänzt.
- **Beleg:** P1 `shot-041`, `shot-042`
- **Nachgeprüft:** nein; abgeglichen an der Datei der Sitzung (`plan`: PDF, 2 Seiten).
- **Status:** offen (Behebung vorbereitet: #167)

### Q1-07 Plan-Ausgabe schneidet ab und kürzt Notizen
- **Art:** Fehler · **Stufe:** mittel
- **Wo:** P5 A-16
- **Getan / Gesehen / Erwartet:** Notiz „Weg frei halten, min. 80 cm“ am unteren Rand: im
  PNG nicht zu sehen; Sofa links halb abgeschnitten; die Notiz „Nebelmaschine außen“ steht als
  „Nebelmaschine a“ (auch im Plan). Erwartet: alles Eingezeichnete im Bild, Notizen ganz
  (LH-16, LH-81).
- **Beleg:** P5 `shot-030`, `sandbox/data/klappe/beispiel/plan.png`
- **Nachgeprüft:** nein; abgeglichen: die Notiz steht im Projekt und in der Darstellungsliste (SVG über
  `klappe-cli`), liegt aber außerhalb des ausgegebenen Ausschnitts. Hängt mit der offenen
  Frage zum Notiztext zusammen.
- **Status:** gewählt (D-51), Behebung vorbereitet: #390

### Q1-08 Die 180°-Prüfung ist nicht auffindbar
- **Art:** vorhanden, aber nicht auffindbar · **Stufe:** mittel
- **Wo:** P2 A-07
- **Getan / Gesehen / Erwartet:** Befehlspalette „180“: „No matches“; „line“: nur Linie und
  Zeitleiste. Achsenspalte und Warnzeichen nicht gesehen. Erwartet: Achse auffindbar,
  Sprung markiert (LH-40, LH-41).
- **Beleg:** P2 `shot-044` bis `shot-046`
- **Nachgeprüft:** nein; abgeglichen an STAND: die Achse (Felder A und B) und der Warn-Chip stehen
  im Shot-Bereich weiter unten, die Spalte „Achsensprung gegen“ als letzte der Tabelle, außerhalb
  des sichtbaren Bereichs (P2 `shot-046`); die Palette kennt sie nicht.
- **Status:** offen

### Q1-09 Kamerablick nur mit gewähltem Shot, folgt nicht dem Schnitt
- **Art:** greift nicht ineinander · **Stufe:** mittel
- **Wo:** P1 A-01
- **Getan / Gesehen / Erwartet:** Abspielen ohne gewählten Shot: „Kein Shot gewählt …“. Mit
  14.1 gewählt zeigt der Blick auch bei 23,5 s noch 14.1, obwohl dort 14.3 läuft. Erwartet: der
  Blick der laufenden Kamera (LH-62, LH-38).
- **Beleg:** P1 `shot-007`, `shot-010` bis `shot-012`
- **Nachgeprüft:** nein; abgeglichen an STAND (Rückfall auf die gewählte Kamera ist offen, Entscheidung des
  Eigentümers); `check/shot-001` zeigt „Kein Shot gewählt“.
- **Status:** offen

### Q1-10 Felder des Shots nur durch Scrollen, Kacheln liegen darüber
- **Art:** Reibung · **Stufe:** mittel
- **Wo:** P2 A-05, P3 A-08/A-10
- **Getan / Gesehen / Erwartet:** Bei sichtbarer Zeitleiste zeigt der Shot-Bereich zwei Zeilen
  Tabelle oder Kacheln und den Anfang der Felder. Die Felder darunter (Inhalt, Requisiten, …)
  kommen nur mit dem Mausrad; dabei liegen die Kacheln über den Feldern. Die Beschreibung heißt
  „Inhalt“. Erwartet: alle Felder eines Shots sichtbar (D-05, D-09).
- **Beleg:** P3 `shot-004`, `shot-018`; P2 `shot-004`, `shot-013`
- **Nachgeprüft:** ja (`check/shot-011`)
- **Status:** offen

### Q1-11 Bogen im Weg nicht auffindbar
- **Art:** vorhanden, aber nicht auffindbar · **Stufe:** niedrig
- **Wo:** P1 A-02
- **Getan / Gesehen / Erwartet:** Weg zwischen Marks gerade; Rechtsklick auf Mark und
  Zeitleiste ohne Wirkung. Erwartet: ein Griff am Weg (LH-21).
- **Beleg:** P1 `shot-029`
- **Nachgeprüft:** nein; abgeglichen an STAND: der Griff erscheint erst am gewählten Weg; nichts weist darauf
  hin.
- **Status:** offen

### Q1-12 Umbenannte Figur: Aktionszeilen bleiben, ohne Hinweis
- **Art:** unverständlich · **Stufe:** niedrig
- **Wo:** P1 K-01
- **Getan / Gesehen / Erwartet:** Wärter → Pförtner: die Repliken heißen PFÖRTNER, die
  Aktionszeile „Der Wärter tritt ein“ bleibt, das Kürzel „W“ auch. Erwartet: Name überall oder
  ein Hinweis.
- **Beleg:** P1 `shot-034`, `shot-037`
- **Nachgeprüft:** nein; abgeglichen am Drehbuch der Sitzung (1 × „Wärter“, 3 × „PFÖRTNER“). So gebaut (STAND,
  D-08); es fehlt der Hinweis.
- **Status:** offen

### Q1-13 Kürzel ist vorbelegt, ohne dass man es merkt
- **Art:** unverständlich · **Stufe:** niedrig
- **Wo:** P1 A-02
- **Getan / Gesehen / Erwartet:** Neue Figur: Name „Bote“, dann ins Kürzelfeld geklickt und
  „B“ getippt; gespeichert ist „BB“. Erwartet: „B“.
- **Beleg:** P1 `shot-022`, `actions.log` (type „Bote“, click, type „B“)
- **Nachgeprüft:** nein; abgeglichen am Code: das Kürzel wird beim Tippen des Namens mit dem
  ersten Buchstaben vorbelegt, die Testperson hat es nicht bemerkt und ergänzt.
- **Status:** offen

### Q1-14 Reihenfolge der Shotliste an zwei Stellen
- **Art:** unverständlich · **Stufe:** niedrig
- **Wo:** P3 A-08
- **Getan / Gesehen / Erwartet:** Shot-Bereich auf „Drehreihenfolge“, Ansicht-Menü zeigt
  gleichzeitig „Schnittfolge“ angehakt. Erwartet: eine Stelle.
- **Beleg:** P3 `shot-006`, `shot-008`
- **Nachgeprüft:** nein; abgeglichen an STAND: die Reihenfolge der Ausgabe ist eine eigene Wahl; das ist
  nirgends gesagt.
- **Status:** offen

### Q1-15 Messen rastet aufs Raster, nicht auf Kanten
- **Art:** Reibung · **Stufe:** niedrig
- **Wo:** P5 A-15
- **Getan / Gesehen / Erwartet:** Ziehen misst nicht, zwei Klicks schon; die Punkte springen auf
  Rasterpunkte; „1,46 m · 59°“ statt Abstand Tischkante–Treppenrand. Erwartet: Fang an Kanten
  wie im CAD (LH-14).
- **Beleg:** P5 `shot-022` bis `shot-025`
- **Nachgeprüft:** nein; abgeglichen an STAND (zwei Klicks, Einrasten wie die anderen Werkzeuge): so gebaut.
- **Status:** gewählt (D-51), Behebung vorbereitet: #390

### Q1-16 Bild-Ausgabe fragt nach Papierformat; Plan dicht überlagert
- **Art:** Reibung · **Stufe:** niedrig
- **Wo:** P5 A-16; P1 A-01
- **Getan / Gesehen / Erwartet:** PNG fragt „A4 / A3“; Lichtkegel und Kamerabereiche überdecken
  Raum und Wege, im Plan wie im Bild. Erwartet: Bild ohne Papierfrage; wählbar, was ins Bild
  kommt.
- **Beleg:** P5 `shot-032`, `plan.png`; P1 `shot-004`
- **Nachgeprüft:** nein; abgeglichen am Code (PNG gehört zu den Formaten mit Papierfrage; `check/shot-001`)
- **Status:** offen

### Q1-17 „Neuer Plan“ übernimmt den Raum
- **Art:** unverständlich · **Stufe:** niedrig
- **Wo:** P5 F-03
- **Getan / Gesehen / Erwartet:** Plan › Neuer Plan „Drehort Hafen“: gleiche Wände, Möbel,
  Notizen; bleibt in Szene 14. Erwartet: ein Weg zu einem neuen Drehort.
- **Beleg:** P5 `shot-037` bis `shot-039`
- **Nachgeprüft:** nein; abgeglichen an STAND: das Set gehört zum Drehort, Drehorte legt Datei › Projekt an;
  im Planmenü steht das nicht.
- **Status:** offen

### Q1-18 Festbrennweiten ohne 85 mm
- **Art:** Reibung · **Stufe:** niedrig
- **Wo:** P2 A-05
- **Getan / Gesehen / Erwartet:** Primes 18, 25, 32, 50, 75, 100, 135 mm; 85 mm nur frei
  eingetippt. Erwartet: 85 mm im Satz.
- **Beleg:** P2 `shot-028`
- **Nachgeprüft:** nein; abgeglichen an `crates/store/camera/standard.toml`
- **Status:** gewählt (D-51), Behebung vorbereitet: #390

### Q1-19 Nach dem Speichern-Dialog wirken Tastenkürzel nicht
- **Art:** Fehler · **Stufe:** mittel
- **Wo:** P1 K-01
- **Getan / Gesehen / Erwartet:** PDF gespeichert, danach Strg+S und Strg+O: nichts, kein
  Dialog, oben weiter „PDF gespeichert“. Erwartet: Speichern und Öffnen-Dialog.
- **Beleg:** P1 `shot-042` bis `shot-044`, `actions.log` Zeilen 113 und 115
- **Nachgeprüft:** ja, frische Sitzung (`check3/shot-006`): nach dem PDF-Dialog öffnet Strg+O
  nichts. Nach einem Klick in den Plan wirken Strg+O und Strg+S (`check2/shot-002`, `-004`).
- **Einordnung:** Die Tastatur kommt nach dem Dialog nicht ins Fenster zurück.
- **Status:** offen (Behebung vorbereitet: #167)

## Nicht bestätigt

- **Eigenes Objekt „Regal“ nicht in der Liste** (P5): steht am Ende der Liste unter „Eigene“,
  erst nach Scrollen sichtbar (`check/shot-010`). Der Tester konnte nicht scrollen (siehe
  Werkzeug).
- **Beschreibungsfeld nicht erreichbar** (P3): mit dem Mausrad erreichbar, heißt „Inhalt“
  (`check/shot-011`); Rest in Q1-10.
- **Ebenen-Liste ohne Schalter, Zeilen „Ausblenden: …“ in der Zeitleiste, Pfeile im Kopf** (P2,
  eigener Blick): in der QA-Sitzung fehlen die Symbole (Icon-Thema); kein Fehler des Programms,
  sondern der Sitzung (Vorfälle).
- **Objektiv von Kamera A nach Änderung an C verändert** (P2, Vermutung): die Daten von A sind
  unverändert (24 mm); ob die Karte den falschen Objektivsatz zeigt, nicht nachgespielt.
- **Neuer Shot verdeckt in der Zeitleiste 14.1 und steht in der Mitte; Setup-Nummer doppelt**
  (P2, P3): nicht nachgespielt.
- **Taste N wechselt in den Zeichenmodus; Palette ohne „Neuer Shot“** (P2): nicht nachgespielt.
- **Statusmeldung mit Pfad abgeschnitten; Plan-Menü doppelt gezeichnet** (P1, P2, P5): nicht
  nachgespielt; das Menü kann an den fehlenden Symbolen der Sitzung liegen.
- **Kein Ladehinweis nach Klick auf die Beispiel-Kachel** (P1): nicht nachgespielt.

## Nicht getestet

P1: A-03, A-04, A-17, A-18, K-02, F-01. P4: alle (A-11, A-12, F-02). Aus den gespielten
Aufgaben: Zoom im Shot (K-04), Schiene als Objekt unter der Kamera (A-06), Mark um genau 0,5 m
(A-13), Speichern und Neuöffnen (A-10, K-01), PDF-Inhalt der Umbenennung (K-01), SVG.

## Vorfälle (Testumgebung)

- Keine: kein Fenster, kein Dialog, keine Datei außerhalb der Sitzungen (alle Dialoge zeigten
  nur den Sandbox-Ordner).
- **Werkzeug:** In der Sitzung fehlen Symbole (Icon-Thema); Knöpfe, die nur ein Symbol haben,
  sind leer oder zeigen ihren Text („Ausblenden: …“). `./qa` kannte für die Tester kein Mausrad
  (es geht mit `click --button 4/5`, stand aber nicht im Briefing); zwei vermeintliche Funde
  gehen darauf zurück. Beides vor Runde 2 beheben.

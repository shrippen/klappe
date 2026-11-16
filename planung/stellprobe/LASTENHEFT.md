# Lastenheft: Stellprobe

Modul der Film-Vorproduktionssuite **Klappe**. Stand: 2026-11-16, Entwurf.

Das Lastenheft beschreibt, *was* Stellprobe leisten muss. *Wie* es umgesetzt wird,
steht in [`../KONZEPT.md`](../KONZEPT.md) und später im Pflichtenheft.

## 1. Ausgangslage

Für die Planung eines Films (zuerst: Wartburgfilm) werden Floorplans gebraucht:
Draufsichten eines Drehorts mit Set, Figuren, Kameras, Licht und deren Bewegungen.
Bisher wurde dafür Hollywood Camera Work *Shot Designer* genutzt. Es läuft nicht unter
Linux. Stellprobe ersetzt es als natives Linux-Programm.

Stellprobe ist eine Clean-Room-Neuentwicklung mit eigenen Verbesserungen. Funktionsumfang,
Schwächen des Vorbilds und die Clean-Room-Regel: Anhang A.

## 2. Ziel

Eine Szene so planen, dass vor dem Dreh feststeht:

- wo Figuren stehen und wie sie sich bewegen,
- wo welche Kamera mit welchem Objektiv steht und wie sie sich bewegt,
- welche Einstellungen (Shots) daraus entstehen,
- wo Licht steht.

Ergebnis ist ein Plan, den Regie, Kamera, Licht und Ausstattung am Set lesen können,
und eine Shotliste, die an die Produktionsplanung der Suite übergeben wird.

## 3. Einbettung in Klappe

```
             Klappe (Suite)
  ┌────────────┬─────────────┬──────────────┐
  │ Drehbuch   │ Stellprobe  │ Planung      │
  │ (später)   │ (zuerst)    │ (später)     │
  └─────┬──────┴──────┬──────┴──────┬───────┘
        │   gemeinsames Datenmodell  │
        └── Szenen, Figuren, Drehorte, Shots ──┘
```

- Stellprobe ist allein nutzbar, ohne die anderen Module.
- Szenen, Figuren und Drehorte sind von Anfang an so angelegt, dass Drehbuch und
  Planung sie später übernehmen, statt eigene Kopien zu führen.
- Shots aus Stellprobe sind die Grundlage für Drehplan und Dispo.

## 4. Nutzer

| Rolle | Nutzt Stellprobe für |
|---|---|
| Regie | Auflösung der Szene, Blocking, Achsen |
| Kamera (DoP) | Kamerapositionen, Objektive, Bewegungen |
| Licht (Oberbeleuchter) | Lampenpositionen |
| Regieassistenz | Shotliste, Drehreihenfolge |
| Ausstattung | Set-Aufbau |

Hauptnutzer ist eine Person (Regie) am Desktop. Die anderen lesen meist nur (Ausdruck, PDF).

## 5. Begriffe

| Begriff | Bedeutung |
|---|---|
| Projekt | Ein Film. Enthält Drehorte, Figuren, Szenen. |
| Drehort | Ort mit Grundriss, z. B. Rittersaal der Wartburg. |
| Szene | Szene aus dem Drehbuch; spielt an einem Drehort. |
| Floorplan | Draufsicht einer Szene: Set, Figuren, Kameras, Licht. |
| Mark | Position einer Figur oder Kamera zu einem Zeitpunkt (1, 2, 3 …). |
| Blocking | Abfolge der Marks und Wege der Figuren. |
| Setup | Eine Kameraposition mit Aufbau; ein Setup kann mehrere Shots liefern. |
| Shot | Einstellung: Kamera, Objektiv, Größe, Bewegung, Inhalt. |
| Achse | 180°-Linie zwischen zwei Figuren. |

## 6. Funktionale Anforderungen

Priorität: **M** = Muss (erste Version), **S** = Soll, **K** = Kann (später).

### 6.1 Projekt und Szenen

| Nr | Anforderung | Prio |
|---|---|---|
| LH-01 | Projekte anlegen, öffnen, speichern; mehrere Szenen je Projekt. | M |
| LH-02 | Drehorte anlegen und mehreren Szenen zuordnen; Set eines Drehorts wird wiederverwendet. | M |
| LH-03 | Figuren projektweit anlegen (Name, Farbe, Kürzel) und in Szenen verwenden. | M |
| LH-04 | Varianten eines Floorplans (z. B. „Plan A / Plan B“) innerhalb einer Szene; Schnappschuss eines Stands, zu dem man zurückkehren kann. | S |
| LH-05 | Automatisches Speichern; kein Datenverlust bei Absturz. | M |
| LH-06 | Rückgängig / Wiederholen ohne feste Grenze innerhalb einer Sitzung. | M |
| LH-07 | Vorlagen für typische Auflösungen (z. B. Schuss-Gegenschuss, Dreieck, Plansequenz); eigene Vorlagen aus einem Plan speichern. | S |
| LH-08 | Auswahl per Klick, Rechteck und Lasso; Auswahl gemeinsam verschieben, drehen, kopieren. | M |
| LH-09 | Ein neuer Floorplan enthält schon die Figuren, die in dieser Szene des Drehbuchs sprechen: nebeneinander am Rand, ohne Wege. Ohne Drehbuch, oder für eine Figur, die noch keiner Projektfigur zugeordnet ist, bleibt der Plan ohne sie. | S |

### 6.2 Set und Grundriss

| Nr | Anforderung | Prio |
|---|---|---|
| LH-10 | Wände, Türen, Fenster, Treppen zeichnen. | M |
| LH-11 | Bibliothek von Objekten (Tisch, Stuhl, Bett, Fahrzeug, Pferd …), skalier- und drehbar. | M |
| LH-12 | Eigene Objekte anlegen und in der Bibliothek ablegen; Bibliotheken als Dateien weitergeben. | M |
| LH-13 | Bild als Hintergrund (Grundriss, Luftbild, Foto; nur Bildformate, kein PDF und kein Video); Maßstab über eine bekannte Strecke kalibrieren. | M |
| LH-14 | Maßstabsgetreues Arbeiten in Metern; Raster, Einrasten, Messwerkzeug. | M |
| LH-15 | Ebenen (Set, Figuren, Kamera, Licht, Notizen) ein- und ausblenden, sperren und in Stufen abblenden (blasser, damit eine andere Ebene hervortritt). | M |
| LH-16 | Beschriftungen und Notizen frei im Plan. | M |
| LH-17 | Höhenangaben für Objekte (für Kamerahöhe und Sichtlinien). | S |
| LH-18 | Bilder (Foto, Skizze) als frei platzierbare Objekte. | S |
| LH-19 | Freies Zeichnen: Linien, Pfeile, Formen, Flächen. | M |

### 6.3 Figuren und Blocking

| Nr | Anforderung | Prio |
|---|---|---|
| LH-20 | Figur platzieren mit Position und Blickrichtung. | M |
| LH-21 | Bewegungswege mit nummerierten Marks; Wege als Kurven. | M |
| LH-22 | Zustand je Mark: stehend, sitzend, liegend. | S |
| LH-23 | Statisten und Gruppen als ein Element. | S |
| LH-24 | Requisiten, die von Figuren bewegt werden, mit eigenem Weg. | K |
| LH-25 | Größe von Figuren und Objekten änderbar (Kind, Pferd, Kutsche). | S |
| LH-26 | Auftritt und Abgang: Figur blendet an einem Mark ein oder aus, statt hinzulaufen. | S |

### 6.4 Kameras

| Nr | Anforderung | Prio |
|---|---|---|
| LH-30 | Mehrere Kameras (A, B, C …) je Szene. | M |
| LH-31 | Bildwinkel aus Brennweite und Sensorformat berechnen und als Sichtkegel zeigen. | M |
| LH-32 | Kamerakörper und Objektive als Voreinstellung (z. B. Sensor Super 35, Vollformat; Festbrennweiten, Zoom). | M |
| LH-33 | Kamerabewegung mit Marks: Schwenk, Fahrt (Dolly, Schiene), Kran, Handkamera, Steadicam/Gimbal, Zoom. | M |
| LH-34 | Kamerahöhe und Neigung je Mark. | S |
| LH-35 | Schienen und Kranarme als Objekte im Plan (Länge, Platzbedarf). | S |
| LH-36 | Kamerablick: Vorschau dessen, was die Kamera sieht (vereinfachte 3D-Sicht mit Figuren als Platzhalter). | S |
| LH-37 | Kamera an Figur koppeln: wird die Figur verschoben, folgt die Kamera und behält Abstand und Winkel. | S |
| LH-38 | Bildformat und Seitenverhältnis je Kamera (z. B. 2,39:1, 1,85:1, 16:9); bestimmt Sichtkegel und Kamerablick. | M |

### 6.5 Achse und Anschlüsse

| Nr | Anforderung | Prio |
|---|---|---|
| LH-40 | 180°-Achse zwischen zwei Figuren anzeigen. | M |
| LH-41 | Warnung, wenn ein Shot die Achse springt, gegenüber anderen Shots derselben Szene. | S |
| LH-42 | Achse folgt den Figuren über die Marks. | S |

### 6.6 Licht

| Nr | Anforderung | Prio |
|---|---|---|
| LH-50 | Lampen platzieren: Typ, Richtung, Abstrahlwinkel. | M |
| LH-51 | Lampen-Bibliothek (Stufenlinse, LED-Panel, Tube, Ballon, Reflektor, Floppy …), Leistung, Farbtemperatur. | S |
| LH-52 | Natürliches Licht: Fenster, Sonnenstand nach Ort, Datum, Uhrzeit. | K |

### 6.7 Ablauf und Wiedergabe

| Nr | Anforderung | Prio |
|---|---|---|
| LH-60 | Szene abspielen: Figuren und Kameras bewegen sich synchron von Mark zu Mark. | M |
| LH-61 | Zeitleiste zum Springen und Feinjustieren der Zeitpunkte. | S |
| LH-62 | Kamerablick (LH-36) läuft bei der Wiedergabe mit. | K |
| LH-63 | Tempo je Weg einstellbar (gehen, rennen, Pause an einem Mark). | M |
| LH-64 | Tonspur (z. B. gelesener Dialog) zur Wiedergabe, für ein einfaches Animatic. | K |

### 6.8 Shots und Shotliste

| Nr | Anforderung | Prio |
|---|---|---|
| LH-70 | Shots im Plan oder in der Shotliste anlegen; beide Ansichten sind wechselseitig verknüpft. Ein Setup kann mehrere Shots liefern. | M |
| LH-71 | Felder je Shot: Nummer, Kamera, Objektiv, Einstellungsgröße (Weit bis Detail), Perspektive, Bewegung, Beschreibung, Dauer, Requisiten, Technik, Team, Notiz. | M |
| LH-72 | Shotliste der Szene; Reihenfolge nach Schnitt und nach Drehreihenfolge (Setups gruppiert). | M |
| LH-73 | Shot im Plan markieren hebt Kamera, Weg und Sichtkegel hervor. | M |
| LH-74 | Storyboard-Bild je Shot (importiert oder skizziert). | K |
| LH-75 | Standbild aus dem Kamerablick als Storyboard-Bild übernehmen. | K |
| LH-76 | Shot-Versionen (12, 12A, 12B) mit Umnummerieren; Nummernschema je Projekt wählbar: Standard Szene und laufende Nummer (12.3); daneben Schemata mit Kamerakennung oder Mehrkamera-Gruppe, die das Pflichtenheft festlegt. Kein Zeichen hat zwei Bedeutungen: Nutzt ein Schema Buchstaben für Kameras, stehen Versionen anders abgesetzt. | S |
| LH-77 | Shot von seinem Setup lösen und einem anderen zuordnen. | S |

### 6.9 Ausgabe

| Nr | Anforderung | Prio |
|---|---|---|
| LH-80 | Druck und PDF: Floorplan mit Legende und Maßstab, Shotliste; A4 und A3. | M |
| LH-81 | Floorplan als PNG und SVG. | M |
| LH-82 | Shotliste als CSV. | M |
| LH-83 | Plan je Mark als Bildfolge (Blocking zum Nachlesen am Set). | S |
| LH-84 | Wiedergabe als Video exportieren. | K |
| LH-85 | Shotliste als Tabelle (ODS, XLSX). | S |
| LH-86 | 3D-Ansicht und Kamerablick als PNG; Kamerablick als Vektorgrafik im PDF. | S |
| LH-87 | Ausgabe mit einer Seite je Shot oder je Kamera: deren Körper, Sichtkegel und Weg kräftig, Set und Figuren blass. | S |

### 6.10 Plan und 3D

Plan und 3D sind zwei Ansichten desselben Sets, keine zwei Betriebsarten. Wer im Plan
zeichnet, legt die Grundlagen für 3D schon mit an. Umsetzung:
[`3D-KONZEPT.md`](3D-KONZEPT.md).

| Nr | Anforderung | Prio |
|---|---|---|
| LH-90 | Raumhöhe je Set; Höhe je Wand (ohne Angabe: Raumhöhe); Brüstung und Sturz je Tür und Fenster. | S |
| LH-91 | Jedes Element hat beim Zeichnen schon eine Höhe aus Vorgaben (Raumhöhe, Bibliothek, Haltung der Figur, Art der Öffnung). Ein Objekt steht auf dem Objekt darunter (Stuhl auf Podest) oder auf einer eigenen Unterkante. | S |
| LH-92 | Freie Form: Umriss frei klicken; sie ist ein Objekt mit Höhe, die ergänzt werden kann. | S |
| LH-93 | Höhen ergänzen: der Plan zeigt Höhen und markiert fehlende; ein Befehl führt nacheinander durch alle Objekte ohne Höhe, die Höhe wird getippt. | S |
| LH-94 | 3D-Ansicht des Sets mit Figuren und Kameras, mit einer Taste zum Plan umschaltbar. Dieselben Werkzeuge, Auswahl und Rückgängig wie im Plan; Drehen, Schieben und Zoomen mit Maus und Touchpad. | S |
| LH-95 | Höhe in Plan und 3D mit der Maus ziehen; Maße per Tastatur: Länge beim Zeichnen von Wänden, Breite und Tiefe eines Raums, Höhe der Auswahl. | S |
| LH-96 | Raum-Werkzeug: ein aufgezogenes Rechteck ergibt vier Wände. | S |
| LH-97 | Bibliotheksobjekte haben in 3D eine einfache Bauform (Tisch mit Beinen, Treppe mit Stufen); neue Bauformen sind Daten, keine Programmänderung. | S |
| LH-98 | Schnitt: Wände in der 3D-Ansicht auf Brusthöhe kappen, damit man in den Raum sieht. | S |
| LH-99 | Blick durch eine Kamera des Plans in der 3D-Ansicht; der Kamerablick (LH-36) zeigt Verdeckung richtig. | S |

### 6.11 Bedienung und Bearbeiten

Aus der Beobachtung von Shot Designer (Anhang A), auf Wunsch des Eigentümers vollständig übernommen (2026-11-16). Jede Funktion hat einen Weg mit der Maus und einen mit der Tastatur (NF-06).

| Nr | Anforderung | Prio |
|---|---|---|
| LH-100 | Kontextmenü an jedem Element im Plan (Figur, Mark, Weg, Kamera, Objekt, Wand) und auf leerer Fläche, mit denselben Befehlen wie Taste und Befehlspalette und der Taste daneben; öffnet auch mit Menütaste und Umschalt+F10. | S |
| LH-101 | Hinweiszeile beim aktiven Werkzeug: was der nächste Klick tut; Esc bricht ab. | S |
| LH-102 | Tooltip mit Name und Taste an jedem Knopf und an den Griffen im Plan. | S |
| LH-103 | Knöpfe Rückgängig und Wiederholen sichtbar; der Tooltip nennt den Schritt. | S |
| LH-104 | Nächster Mark aus der Auswahl: Figur oder Kamera gewählt, Taste oder Kontextmenü, dann ein Klick setzt den nächsten Mark, ohne Werkzeugwechsel (Kamera: Rig Dolly). | S |
| LH-105 | Mark davor setzen: ein Mark vor dem ersten, woher die Figur kommt; die Zeiten rücken nach. | S |
| LH-106 | Nummer an jedem Mark (1, 2, 3 je Figur) im Plan. | S |
| LH-107 | Wegstück verschieben: ein Stück Weg mit beiden Enden und Knicken auf einmal ziehen oder mit Pfeiltasten schieben. | S |
| LH-108 | Weg gerade oder weich: je Weg umschalten zwischen Kurve durch die Knicke und geraden Strecken (Schiene, Flur). | S |
| LH-109 | Achse von der Figur aus: Figur gewählt, „Achse zu …“, Klick auf die zweite Figur setzt das Achsenpaar (LH-40). | S |
| LH-110 | Ein neuer Shot öffnet Name und Beschreibung zum Tippen, Enter schließt ab. | S |
| LH-111 | Kennung des Shots in der Farbe seiner Kamera in Shotliste und Kacheln, Name dazu (Farbe nie allein). | S |
| LH-112 | Lange Beschreibung eines Shots aufklappbar als zweite Zeile am gewählten Shot. | S |
| LH-113 | Kamerablick als kleines Bild neben dem Kamerakörper im Plan, abschaltbar (LH-36). | S |
| LH-114 | Art eines Objekts tauschen: Ort, Drehung und Ebene bleiben. | S |
| LH-115 | Objekt spiegeln (links und rechts). | S |
| LH-116 | Maße eines Objekts auf die Bibliothek zurücksetzen. | S |
| LH-117 | Bibliothek um Crew-Plätze (Tonangel, Monitorplatz, Regie), Fahrzeuge (Bus, LKW, Motorrad), Tier (Hund) und kleine Requisiten (Telefon, Laptop) erweitern. | S |
| LH-118 | Beschriftung an Objekt, Figur oder Mark gebunden; sie folgt beim Verschieben. | S |
| LH-119 | Gespeicherte Ansichten: benannte Auswahl sichtbarer Kameras, Figuren und Ebenen; Grundlage der Ausgabe je Shot oder Kamera (LH-87). | S |
| LH-120 | Skizzenstift: Freihand über dem Plan auf eigener Ebene, ein Befehl löscht alle Skizzen. | S |
| LH-121 | Schwenken auch mit gedrückter mittlerer Maustaste (Leertaste gibt es schon). | S |
| LH-122 | Ein Werkzeug zum Setzen bleibt nach dem Klick aktiv für weitere Elemente; Esc wechselt zur Auswahl. | S |
| LH-123 | Das Werkzeug merkt die zuletzt gewählte Figur, Kamera, Objektart und Lampe. | S |

## 7. Nicht-funktionale Anforderungen

| Nr | Anforderung | Prio |
|---|---|---|
| NF-01 | Läuft nativ unter Linux (KDE Plasma, Wayland), zuerst. Windows und macOS folgen mit derselben Codebasis; nichts Linux-Eigenes außerhalb einer Plattformschicht. | M |
| NF-02 | Arbeitet vollständig offline; keine Konten, keine Cloud. | M |
| NF-03 | Offenes, dokumentiertes Dateiformat; textbasiert, damit Projekte in Git versioniert und verglichen werden können. | M |
| NF-04 | Oberfläche nach dem Design System Kante. | M |
| NF-05 | Flüssige Bedienung bei mindestens 50 Figuren, 10 Kameras, 300 Set-Objekten je Floorplan. | M |
| NF-06 | Bedienung mit Maus und Tastatur; Tastenkürzel für alle häufigen Werkzeuge. Jede Funktion hat beide Wege: mit der Maus (Klick, Ziehen, Kontextmenü) und mit der Tastatur (Taste oder Befehlspalette); kein Befehl nur über Hover, Rechtsklick oder eine Taste allein, damit Touch (NF-07) später ohne Umbau dazukommt. | M |
| NF-07 | Stift- und Touch-Bedienung (Tablet am Set); später; die erste Version zielt nicht auf Touch (E-01 in `../KONZEPT.md`). | K |
| NF-08 | Sprachen: Deutsch und Englisch. | M |
| NF-09 | Fachbegriffe nach deutscher Filmpraxis, englische Begriffe in der englischen Fassung. | M |
| NF-10 | Alle Funktionen ohne Konto und ohne Bezahlgrenze; Abgleich zwischen Geräten über Git oder beliebigen Dateiabgleich. | M |

## 8. Abgrenzung

Nicht Teil von Stellprobe:

- Drehbuch schreiben, Drehplan, Dispo, Budget (andere Module der Suite),
- vollständige 3D-Previsualisierung mit Animation und Rendering,
- Zusammenarbeit mehrerer Personen gleichzeitig am selben Plan,
- Import von Shot-Designer-Dateien (geschlossenes Format; Auslesen widerspräche der Clean-Room-Regel),
- Cloud-Dienst oder Konten.

## 9. Abnahme

Die erste Version ist abgenommen, wenn

1. eine Dialogszene des Wartburgfilms mit zwei Figuren, zwei Kameras, drei Marks je
   Figur und mindestens sechs Shots vollständig geplant werden kann,
2. der Grundriss des Drehorts als Bild hinterlegt und maßstabsgetreu kalibriert ist,
3. die Wiedergabe die Bewegungen korrekt zeigt,
4. PDF mit Floorplan und Shotliste lesbar auf A4 gedruckt werden kann,
5. alle Muss-Anforderungen erfüllt sind.

## 10. Entscheidungen

Die offenen Fragen sind beantwortet (2026-11-13): Kein Tablet in der ersten Version, keine
Übernahme alter Shot-Designer-Pläne, Kamerablick bleibt Soll, PDF reicht zur Weitergabe,
Lizenz MIT. Alle Entscheidungen: [`../KONZEPT.md`](../KONZEPT.md), Abschnitt 7.

Plan und 3D (2026-11-15): eine Szene für Plan, 3D-Ansicht und Kamerablick; eigener
Renderer statt Qt Quick 3D (GPLv3); freie Formen sind Objekte; Körpergröße je Figur später.
Einzelheiten: [`3D-KONZEPT.md`](3D-KONZEPT.md), Abschnitt 9.

## Anhang A: Analyse Shot Designer

### Clean-Room-Regel

Stellprobe ist eine Neuentwicklung. Übernommen wird nur, *was* Shot Designer
für den Nutzer leistet, nie *wie* es das tut.

- Quelle sind ausschließlich die öffentlichen Produktseiten von Hollywood Camera Work
  (Liste unten). Kein Disassemblieren, kein Mitschnitt des Netzverkehrs, kein Auslesen
  der Dateiformate, keine Übernahme von Grafiken, Symbolen, Texten oder Vorlagen.
- Wer Stellprobe programmiert, arbeitet nach Lastenheft und Pflichtenheft, nicht mit
  Shot Designer daneben.
- Namen, Symbole, Objektbibliothek und Dateiformat sind eigene Entwürfe.
- Ausnahme auf Auftrag des Eigentümers (2026-11-16, Wortlaut im Kommentar zu #197): Seine Kopie
  (freie Fassung) wurde in einer unsichtbaren Sitzung bedient, um Bedienung und Tempo beim Blocking zu
  beobachten. Daraus steht hier nur Verhalten in eigenen Worten (Abschnitt „Shot Designer
  in der Bedienung“); nichts sonst aus der Liste oben wurde getan.

Gelesene Seiten (2026-11-13):

- hollywoodcamerawork.com/shot-designer.html
- hollywoodcamerawork.com/shot-designer-download.html
- hollywoodcamerawork.com/shot-designer-videos.html (nur Titelliste der Videos)
- hollywoodcamerawork.com/shot-designer-sync.html
- hollywoodcamerawork.com/shot-designer-testimonials.html (teilweise)

Ein Handbuch ist dort nicht öffentlich verlinkt. Was nicht auf den Seiten steht, ist
unten als *vermutet* gekennzeichnet.

### Funktionsumfang

| Bereich | Shot Designer | Anforderung |
|---|---|---|
| Szenen | Szenen in Ordnern; mehrere Seiten je Szene | LH-01, LH-04 |
| Schnappschüsse | Stand einfrieren und zurückkehren („Scene Freeze“, Pro) | LH-04 |
| Vorlagen | fertige Kamera-Aufbauten und eigene Vorlagen | LH-07 |
| Set | Wandwerkzeug mit Türen und Fenstern; Objektbibliothek | LH-10, LH-11 |
| Hintergrund | Grundriss oder Kartenausschnitt als Bild | LH-13 |
| Bilder als Objekte | ja | LH-18 |
| Auswahl | Lasso, Einrasten | LH-08, LH-14 |
| Figuren | platzieren, auf Wegen bewegen | LH-20, LH-21 |
| Kamera folgt Figur | Kameras bewegen sich mit, wenn Figuren verschoben werden | LH-37 |
| Kameras | Marks, Bewegung in der Animation; Objektiv im Sucher | LH-30–LH-33 |
| Sichtkegel, Sensor | nicht beschrieben, *vermutet* vorhanden | LH-31, LH-32 |
| Kran, Schiene | nur in Nutzerberichten, kein eigenes Objekt genannt | LH-35 |
| 180°-Achse | nur in Nutzerberichten, kein eigenes Werkzeug genannt | LH-40–LH-42 |
| Licht | Lichtsymbole, Lichtplan | LH-50, LH-51 |
| Animation | Zeitleiste; Einblenden statt Bewegen; Tempo je Objekt | LH-60–LH-63, LH-26 |
| Sucher | Blick aus der Kamera mit Kameraformaten und Seitenverhältnis | LH-36, LH-38 |
| 3D | nicht vorhanden, angekündigt | LH-36, LH-90–LH-99 |
| Shotliste | wechselseitig mit dem Plan verknüpft; Versionen; Drehreihenfolge; Felder Typ, Objektiv, Requisiten, Technik, Team; Shot lösen | LH-70–LH-77 |
| Storyboard | Schlüsselbilder je Kamera, aus dem Sucher | LH-74, LH-75 |
| Ausgabe | PDF, JPG, Excel, E-Mail (Pro) | LH-80–LH-85 |
| Abgleich | Cloud-Sync und Team-Ordner (Pro, Konto nötig) | bewusst nicht, NF-02 |
| Plattformen | Windows, macOS, iOS, Android; **kein Linux** | NF-01 |
| Preis | frei für eine Szene; Pro 19,99 € | frei, ohne Grenzen |

### Schwächen laut Nutzern und Hersteller

1. Kein 3D. Angekündigt, die Entwicklung ruht.
2. Objektbibliothek zu klein; eigene Objekte fehlen (häufigster Wunsch).
3. Kein freies Zeichnen; Figuren nicht in der Größe änderbar.
4. Lichtwerkzeuge und Lampenbibliothek dürftig.
5. Zeitleiste grob; Timing nicht am Drehbuch ausrichtbar.
6. Kein Ton für ein Animatic.
7. Keine Verbindung zum Drehbuch.
8. Funktionen für Dateiverwaltung, Export und Abgleich nur in der Bezahlversion,
   Abgleich nur über Konto und Cloud.
9. Geschlossenes Dateiformat.

### Shot Designer in der Bedienung (beobachtet, in eigenen Worten)

Beobachtet wurde die Fassung ohne Pro-Lizenz. Wo das der Tabelle „Funktionsumfang“ (nach den
Herstellerseiten) widerspricht, etwa bei Lasso, Kamera folgt Figur, Tempo und Sichtkegel, gilt
die Tabelle als Angabe des Herstellers und dieser Abschnitt als das, was in der Bedienung zu
finden war.

- Die Oberfläche ist für Berührung gebaut: keine Tooltips, kein Rechtsklick, kein Menü auf
  leerer Fläche. Ein Element öffnet sein Menü erst beim zweiten Tippen, ohne Hinweis.
- Eine neue Szene beginnt mit zwei Figuren; eine weitere Figur kostet vier Klicks.
- Eine Position mehr kostet drei Klicks (Figur, „hin zu“, Ziel); die Blickrichtung folgt dem
  Weg. Kurven entstehen über einen Kontrollpunkt. Dauer und Tempo fanden sich nicht.
- Eine Kamera kommt erst in die Shotliste, wenn sie eine Beschreibung bekommt; das steht
  nirgends. Brennweite, Bildwinkel und eine Kamera, die einer Figur folgt, fanden sich nicht.
- Ebenen lassen sich je Art sperren und in Stufen blasser stellen. Seiten fassen Auswahlen
  zusammen, auch automatisch: eine Seite je Kamera oder je Shot.
- Ziehen auf leerer Fläche schwenkt; eine Mehrfachauswahl gibt es nicht.
- Speichern und Ausgabe sind Teil der bezahlten Fassung.

### Verbesserungen in Stellprobe

| Schwäche | Stellprobe | Anforderung |
|---|---|---|
| 1 | Kamerablick in vereinfachtem 3D, läuft bei der Wiedergabe mit; 3D-Ansicht auf demselben Set wie der Plan, mit denselben Werkzeugen | LH-36, LH-62, LH-90–LH-99 |
| 2 | Eigene Objekte als Muss; Bibliothek als offene Dateien teilbar | LH-12 |
| 3 | Freies Zeichnen, Formen; Figuren und Objekte skalierbar | LH-19, LH-25 |
| 4 | Lampenbibliothek mit Leistung und Farbtemperatur; Sonnenstand | LH-51, LH-52 |
| 5 | Zeitleiste mit Feinjustierung; Tempo je Weg | LH-61, LH-63 |
| 6 | Tonspur zur Wiedergabe | LH-64 |
| 7 | Gemeinsames Datenmodell der Suite: Szenen und Figuren aus dem Drehbuch | Abschnitt 3 |
| 8 | Alles frei, kein Konto; Abgleich über Git oder beliebigen Dateiabgleich | NF-02, NF-03 |
| 9 | Offenes, textbasiertes Format | NF-03 |
| – | Achsenwarnung als eigenes Werkzeug statt Erfahrungswissen | LH-41 |
| – | Sichtkegel aus Sensor und Brennweite, sichtbar im Plan | LH-31 |
| – | Plan je Mark als Bildfolge für das Set | LH-83 |
| – | Jeder Befehl mit Maus und Tastatur, mit Tooltip und Kontextmenü; kein verstecktes zweites Tippen | NF-06 |

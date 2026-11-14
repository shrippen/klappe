# Arbeitsstand

Stand: 2026-11-14.

## Phase 0 (KONZEPT.md, Abschnitt 6)

| Schritt | Stand |
|---|---|
| Repo-Grundlage: Workspace, CI, Regeln | fertig (#1) |
| Kern: Modell, Optik, Bewegung, Achse, Darstellungsliste | fertig (#1) |
| Fountain-Parser | fertig (#2) |
| Projektordner lesen und schreiben | fertig (#3) |
| Export SVG und PDF aus der Darstellungsliste | fertig (#4) |
| Floorplan als PNG (eigener Rasterer, Liberation Sans) und SVG (LH-81): App-Knöpfe und `klappe-cli export` | fertig |
| Sitzung (`services`): Szene, Zeit, Shot bearbeiten, Rückgängig, Ansichten | fertig (#5) |
| TUI-Editor (`klappe-tui`), schreibt nur das Drehbuch | fertig (#6, #8) |
| TUI nach D-10: Kante-Farben (aktive Szene gelber Balken, Szenenkopf ohne `#12#` mit Nummer rechts), Marken für Shots und Marks rechts an der Zeile, Texte aus dem Katalog (`tui.*`, `--language`), dichte Ansicht (Strg+B Szenenliste, unter 100 Spalten aus; Strg+P Szenenwechsel mit Suche; Kopfzeile mit Shots und Figuren, Statuszeile mit Modus, Szene x/y, Zeile:Spalte) | erste Fassung; offen: Marken zeigen den gespeicherten Stand (nach Strg+S neu gelesen), Marken nur, wenn die Zeile sie neben dem Text unterbringt |
| GUI-Gerüst (cxx-qt, Kirigami, Kante), Layout E-14: Drehbuch, Plan, Shot, Zeitleiste, ausklappbar | erste Fassung (#7, Kante #170) |
| App in der CI (Arch, QML-Tests, offscreen-Screenshots) | fertig (#7) |
| Flatpak und Windows-Build der App | offen |
| Werkzeuge zum Zeichnen (Wand, Objekt, Figur, Kamera, Licht) | erste Fassung |
| Automatisches Speichern (LH-05) | fertig |
| Projekte, Szenen, Drehorte, Figuren verwalten (LH-01 bis LH-03): Startseite mit zuletzt geöffneten Projekten, Projektdialog mit Listen, Rückgängig | erste Fassung; offen: Szenen und Figuren umsortieren, Szenennummern automatisch vergeben und beim Drehbuch-Umbau nachführen (DB-33) |
| Hintergrundbild mit Maßstab (LH-13) | erste Fassung; offen: PDF als Hintergrund |
| Messwerkzeug (LH-14): Taste D, zwei Klicks setzen die Strecke, gestrichelte Linie mit Endpunkten und Länge (m) folgt dem Zeiger, ein dritter Klick beginnt neu, Esc und Wechsel von Werkzeug, Szene oder Plan verwerfen; Einrasten wie bei den anderen Werkzeugen; Werkzeugzustand der Sitzung, nicht im Plan, nicht im Export, kein Rückgängig | erste Fassung; offen: Winkel in der Statuszeile |
| Notizen im Plan (LH-16): Werkzeug, Ebene, Export | erste Fassung; offen: Doppelklick zum Bearbeiten (F2 oder Klick mit dem Notiz-Werkzeug), Textform statt Marker mit Beschriftung |
| Freies Zeichnen (LH-19): Linie, Pfeil, Rechteck, Ellipse, Fläche je Szenenplan, Farbe, Strichart, Füllung, Ebene, Export | erste Fassung; offen: Stil einer vorhandenen Zeichnung ändern, Punkte und Ecken ziehen, Drehen, Zeichnungen in der Legende, Werkzeugleiste bei schmalen Fenstern |
| Ebenen sperren (LH-15): Schalter „gesperrt“ je Ebene, nichts wählbar oder änderbar | erste Fassung; die Sperre gilt für die Sitzung und steht nicht im Projekt |
| Mehrfachauswahl (LH-08): Klick, Umschalt+Klick, Rechteck (Ziehen auf leerer Fläche, Umschalt fügt hinzu); gemeinsam verschieben, drehen (R, um die Mitte der Auswahl), löschen (Entf), kopieren (Strg+D, Kopien werden gewählt), je ein Rückgängig-Schritt; gesperrte Ebenen nicht wählbar, ein gesperrtes Element lehnt die Gruppenänderung ab | erste Fassung; offen: Lasso; Griffe nur per Klick wählbar, nicht per Rechteck; Figuren, Marks und Griffe lassen sich nicht kopieren; Zeichnungen, Notizen und Griffe schwenken beim Drehen nur mit, drehen sich nicht; ein Linksziehen auf leerer Fläche schwenkt die Ansicht nicht mehr (Mitteltaste oder Leertaste) |
| Wege als Kurven (LH-21): `via` je Mark (quadratische Bézierkurve vom vorigen Mark), gleichmäßiges Tempo, Griff in der Mitte des gewählten Wegs ziehen, Taste G macht ihn gerade | erste Fassung; offen: Kurve mit mehr als einem Knick (Spline), Griff beim Verschieben der Marks mitführen |
| Kamera folgt Figur (LH-37): Verschieben und Drehen eines Marks nimmt die Kamera-Marks im Zeitfenster des Marks mit; Auswahl im Shot-Bereich, Hinweis im Tooltip | erste Fassung; die Kopplung gilt je Kamera für alle Marks im Fenster, bei gesperrter Kamera-Ebene wird die Figurenänderung abgelehnt |
| Objektbibliothek (LH-11, LH-12): eingebaute Bibliothek als Datei (Tisch, Stuhl, Bett, Sofa, Tür, Fenster, Säule, Kamin, Treppe, Auto, Kutsche, Pferd, Baum, Podest), Form Rechteck oder Ellipse, eigene Objekte je Projekt in `bibliothek.toml`, Ablegen aus einem Objekt, Import und Export einer Bibliotheksdatei | erste Fassung; offen: eigene Objekte löschen oder umbenennen, Gruppen in der Bibliothek |
| Größe von Objekten ändern (LH-25): Breite und Tiefe als Felder, Rückgängig, Sperre der Set-Ebene | erste Fassung für Objekte; offen: Figuren, Griffe zum Ziehen |
| Kamerakörper und Objektive als Voreinstellung (LH-32): eingebaute Datei `crates/store/camera/standard.toml` mit Sensorformaten (Super 35, Full Frame, Micro Four Thirds, APS-C, Super 16, Large Format) und Objektivsätzen (Festbrennweiten, Zoom); Kamera-Karte im Plan: Körper, Sensorbreite, Objektiv aus dem Satz oder frei, Seitenverhältnis; Rückgängig, Sperre der Kameraebene | erste Fassung; offen: eigene Voreinstellungen je Projekt, Herstellermodelle |
| Perspektive und Drehreihenfolge der Shotliste (LH-71, LH-72) | fertig |
| Shots anlegen, kopieren, löschen (LH-70): Knöpfe und Taste S im Shot-Bereich (Kamera aus der Plan-Auswahl, Spanne ab Abspielkopf bis zum nächsten Mark), Nummer `Szene.n` fortlaufend, Rückgängig, Kameraebene-Sperre | erste Fassung; offen: Anlegen mit gezogener Spanne in der Zeitleiste, eigene Shotliste als Tabelle mit Zeilenwahl, Shot einem anderen Setup zuordnen (LH-77) |
| Haltung, Pause und Tempo je Mark (LH-22, LH-63): Karte am gewählten Mark mit Haltung (stehend, sitzend, liegend), Pause in s und dem Tempo des ankommenden Wegs (Zahl, nur lesend) mit Knöpfen Gehen (1,4 m/s) und Rennen (4 m/s); Pause und Tempo verschieben die späteren Marks der Spur, Kurven zählen nach Länge; Rückgängig, Sperre der Figuren- und Kameraebene | erste Fassung; das Tempo wird nicht gespeichert, es setzt `at_s`; folgende Kameras behalten ihre Zeiten; offen: Haltung nicht in der Darstellungsliste, Tempo frei eingeben, Zeit in der Zeitleiste ziehen (LH-61) |
| Auftritt und Abgang einer Figur (LH-26): je Figuren-Mark `arrival` (läuft hin, erscheint) und `departure` (bleibt, geht ab), Felder in der Mark-Karte (zweite Zeile), Rückgängig, Sperre der Figurenebene; `figure_at` liefert `None`, solange die Figur nicht da ist; die Darstellungsliste zeichnet sie dann nicht, der Weg zu einem Auftritts-Mark fehlt, die Mark-Nummer trägt `+` (Auftritt) und `-` (Abgang) | erste Fassung; hart ein/aus, kein Ein-/Ausblenden (die Darstellungsliste kennt Deckkraft nur für Bilder); offen: Zeichen am Mark als eigenes Symbol statt `+`/`-` |
| Kamerahöhe und Neigung je Mark (LH-34): Felder Höhe (m, 0 bis 50) und Neigung (Grad, −90 bis 90, positiv blickt nach oben) in der Mark-Karte (zweite Zeile) für den Kamera-Mark am Abspielkopf, ein Rückgängig-Schritt für beide, Sperre der Kameraebene | erste Fassung; nur Daten und Bedienung, der Kamerablick (LH-36) wertet sie noch nicht aus |
| Höhe von Objekten (LH-17): `height_m` je Objekt und je Bibliothekseintrag (eingebaute Objekte mit Höhe, Podest 0,4 m), neues Objekt und „Ablegen“ übernehmen sie, Feld Höhe in der Objekt-Karte (leer = keine Angabe), Tooltip nennt sie; Rückgängig, Sperre der Set-Ebene | erste Fassung; nur Daten und Bedienung, Sichtlinien und Kamerablick werten sie noch nicht aus |
| Varianten eines Floorplans (LH-04): Auswahl der Pläne einer Szene über dem Plan, Neuer Plan (leer), Schnappschuss (Kopie des gezeigten Plans samt Shots, „Plan A (2)“), Umbenennen, Löschen mit Rückfrage; Namen einer Szene und Ids sind eindeutig (ohne Groß-/Kleinschreibung), Rückgängig, die Ansicht bleibt beim Plan; Zurückkehren heißt den früheren Plan wieder wählen | erste Fassung; abgelehnt wird das Löschen des letzten Plans einer Szene und eines Plans mit Shots (sie gingen mit); offen: Pläne umsortieren, Vergleich zweier Pläne, Plan einer anderen Szene kopieren |
| Demo-Modus mit „Studio Weber“ für Screenshots | offen |

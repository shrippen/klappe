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
| GUI-Gerüst (cxx-qt, Kirigami, Kante), Layout E-14: Drehbuch, Plan, Shot, Zeitleiste, ausklappbar | erste Fassung (#7, Kante #170) |
| App in der CI (Arch, QML-Tests, offscreen-Screenshots) | fertig (#7) |
| Flatpak und Windows-Build der App | offen |
| Werkzeuge zum Zeichnen (Wand, Objekt, Figur, Kamera, Licht) | erste Fassung |
| Automatisches Speichern (LH-05) | fertig |
| Projekte, Szenen, Drehorte, Figuren verwalten (LH-01 bis LH-03): Startseite mit zuletzt geöffneten Projekten, Projektdialog mit Listen, Rückgängig | erste Fassung; offen: Szenen und Figuren umsortieren, Szenennummern automatisch vergeben und beim Drehbuch-Umbau nachführen (DB-33) |
| Hintergrundbild mit Maßstab (LH-13) | erste Fassung; offen: PDF als Hintergrund |
| Notizen im Plan (LH-16): Werkzeug, Ebene, Export | erste Fassung; offen: Doppelklick zum Bearbeiten (F2 oder Klick mit dem Notiz-Werkzeug), Textform statt Marker mit Beschriftung |
| Freies Zeichnen (LH-19): Linie, Pfeil, Rechteck, Ellipse, Fläche je Szenenplan, Farbe, Strichart, Füllung, Ebene, Export | erste Fassung; offen: Stil einer vorhandenen Zeichnung ändern, Punkte und Ecken ziehen, Drehen, Zeichnungen in der Legende, Werkzeugleiste bei schmalen Fenstern |
| Ebenen sperren (LH-15): Schalter „gesperrt“ je Ebene, nichts wählbar oder änderbar | erste Fassung; die Sperre gilt für die Sitzung und steht nicht im Projekt |
| Mehrfachauswahl (LH-08): Klick, Umschalt+Klick, Rechteck (Ziehen auf leerer Fläche, Umschalt fügt hinzu); gemeinsam verschieben, drehen (R, um die Mitte der Auswahl), löschen (Entf), kopieren (Strg+D, Kopien werden gewählt), je ein Rückgängig-Schritt; gesperrte Ebenen nicht wählbar, ein gesperrtes Element lehnt die Gruppenänderung ab | erste Fassung; offen: Lasso; Griffe nur per Klick wählbar, nicht per Rechteck; Figuren, Marks und Griffe lassen sich nicht kopieren; Zeichnungen, Notizen und Griffe schwenken beim Drehen nur mit, drehen sich nicht; ein Linksziehen auf leerer Fläche schwenkt die Ansicht nicht mehr (Mitteltaste oder Leertaste) |
| Wege als Kurven (LH-21): `via` je Mark (quadratische Bézierkurve vom vorigen Mark), gleichmäßiges Tempo, Griff in der Mitte des gewählten Wegs ziehen, Taste G macht ihn gerade | erste Fassung; offen: Kurve mit mehr als einem Knick (Spline), Griff beim Verschieben der Marks mitführen |
| Kamera folgt Figur (LH-37): Verschieben und Drehen eines Marks nimmt die Kamera-Marks im Zeitfenster des Marks mit; Auswahl im Shot-Bereich, Hinweis im Tooltip | erste Fassung; die Kopplung gilt je Kamera für alle Marks im Fenster, bei gesperrter Kamera-Ebene wird die Figurenänderung abgelehnt |
| Objektbibliothek (LH-11, LH-12): eingebaute Bibliothek als Datei (Tisch, Stuhl, Bett, Sofa, Tür, Fenster, Säule, Kamin, Treppe, Auto, Kutsche, Pferd, Baum, Podest), Form Rechteck oder Ellipse, eigene Objekte je Projekt in `bibliothek.toml`, Ablegen aus einem Objekt, Import und Export einer Bibliotheksdatei | erste Fassung; offen: eigene Objekte löschen oder umbenennen, Gruppen in der Bibliothek, Höhe (LH-17) |
| Größe von Objekten ändern (LH-25): Breite und Tiefe als Felder, Rückgängig, Sperre der Set-Ebene | erste Fassung für Objekte; offen: Figuren, Griffe zum Ziehen |
| Perspektive und Drehreihenfolge der Shotliste (LH-71, LH-72) | fertig |
| Demo-Modus mit „Studio Weber“ für Screenshots | offen |

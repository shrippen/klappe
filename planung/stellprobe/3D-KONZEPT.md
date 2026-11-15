# Konzept: Plan und 3D auf einer Grundlage

Stand: 2026-11-15, Entwurf. Grundlage: Prototyp [`prototyp-3d/`](prototyp-3d/) und der Code
von heute (`stellprobe/src/sight.rs`, Plan-Zeichenfläche, `model/`).
Entscheidungen des Eigentümers: Abschnitt 9 (getroffen).

## 1. Ziel

Rudimentäres visuelles Blocking in 3D, ohne zweites Programm im Programm:

- Man fängt im Plan an und legt dabei die Grundlagen für 3D schon mit an.
- Was in 3D fehlt (Höhen), ist in Sekunden ergänzt, nicht in Minuten.
- Dieselben Werkzeuge im Plan und in 3D; 3D ist eine Ansicht, kein Modus.
- Fertige Objekte aus der Bibliothek und freie Formen.
- Eigener Renderer, keine Abhängigkeit, MIT (Qt Quick 3D ist GPLv3, `KONZEPT.md`).

Nicht Ziel: Previsualisierung mit Licht, Schatten, Texturen, Animation von Gliedmaßen,
Import von 3D-Modellen (Lastenheft, Abschnitt 8).

## 2. Grundidee: jedes Element ist Grundriss plus Höhe

```
  Element ─┬─► Grundriss (x, y in Metern)   ──► Plan: Umriss, wie heute
           └─► Unterkante + Höhe             ──► 3D: senkrecht hochgezogen (Prisma)

  Höhe leer = Vorgabe:   Wand        ← Raumhöhe des Sets
                         Öffnung     ← Brüstung / Sturz ihrer Art
                         Bibliothek  ← Höhe des Eintrags (beim Setzen übernommen)
                         Figur       ← Haltung (steht, sitzt, liegt)
                         freie Form  ← fehlt; 3D zeigt 0,90 m halbdurchsichtig
  Unterkante leer = steht auf dem größten Objekt unter seiner Mitte, sonst auf dem Boden
```

Damit ist jeder Plan schon ein 3D-Set. Nur freie Formen brauchen eine Angabe, und das
Programm führt durch sie („Höhen ergänzen“, Abschnitt 6).

## 3. Modell (`model/`, `FORMAT.md`)

Vor dem ersten Release: Breaking Changes erlaubt, `FORMAT_VERSION` 1 → 2, Testdaten und
Beispielprojekte werden migriert, keine Kompatibilitätsschicht.

| Typ | Neu | Bedeutung |
|---|---|---|
| `SetPlan` | `wall_height_m: f64` (Standard 2,60) | Raumhöhe; Vorgabe aller Wände |
| `Wall` | `height_m: Option<f64>` | eigene Höhe, sonst Raumhöhe |
| `Opening` | `sill_m: Option<f64>`, `head_m: Option<f64>` | Brüstung, Sturz; leer = Vorgabe der Art (Tür 0/2,10, Fenster 0,90/2,10) |
| `SetObject` | `base_m: Option<f64>` | Unterkante; leer = stapeln |
| `ObjectShape` | `Polygon { points }` | freie Form; Punkte relativ zu `at`, vor der Drehung |
| `LibraryItem` | `form: Option<String>` | Name einer Bauform (Abschnitt 4.2); leer = Quader bzw. Zylinder |

`SetObject.height_m` und die Höhen der Bibliothek gibt es schon (LH-17). Die Höhe einer
Figur bleibt eine Frage der Haltung; Körpergröße je Figur (Kind) später, Abschnitt 9.

## 4. Kern (`stellprobe/`): eine Szene für alles

### 4.1 Höhen an einer Stelle

Neues Modul `stellprobe::height`: `wall_top`, `opening_span`, `object_extent` (Unterkante,
Oberkante, mit Stapeln), `figure_extent`. Heute stehen die Werte als Konstanten in `sight.rs`
(`WALL_HEIGHT_M`, `OBJECT_HEIGHT_M`, `STANDING_M` …); sie ziehen hierher um. Kamerablick,
3D-Ansicht, Tooltips und Karten lesen nur noch dieses Modul.

### 4.2 Szene

`stellprobe::scene` macht aus Set, Plan und Zeitpunkt `t_s` eine Liste von Körpern:

```
  Solid { source: Source, role: Role3, faces: Vec<Face> }     Face = Polygon in 3D + Normale
```

- `source` ist dieselbe Quelle wie in der Darstellungsliste des Plans (`display::Source`):
  ein Klick in 3D trifft dasselbe Element wie ein Klick im Plan.
- `role` ist eine Rolle (Wand, Objekt, freie Form, Figur mit Farbe, Kamera, Auswahl,
  fehlende Höhe), nie eine Farbe. Die Oberfläche bildet sie auf Kante ab, wie beim Plan.
- Wände: Stücke zwischen den Öffnungen, Brüstung und Sturz als eigene Stücke, Enden um die
  halbe Dicke verlängert (Ecken geschlossen).
- Objekte: Prisma aus dem Grundriss; Bibliotheksobjekte mit Bauform aus Teilquadern im
  Einheitswürfel (Tisch = Platte + vier Beine, Treppe = Stufen), skaliert auf Breite,
  Tiefe, Höhe. Die Bauformen stehen als Daten in `store/library/` (wie Objekte heute), eine
  neue Bauform braucht keinen Code.
- Figuren: Achteckprisma als Körper, kleiner Kopf, Nase in Blickrichtung; Pose aus der
  Zeitleiste, wie im Plan.
- Kameras: Gehäuse; der Sichtkegel als Linien kommt aus der Szene, nicht aus der Oberfläche.

### 4.3 Kamera der Ansicht

```
  enum Eye { Top { centre, metres_per_px }, Orbit { target, yaw_deg, pitch_deg, distance_m, fov_deg },
             Shot { camera, t_s } }
```

- `Top` ist der Plan (senkrecht von oben, ohne Perspektive).
- `Orbit` ist die freie 3D-Ansicht.
- `Shot` ist der Blick durch eine Plan-Kamera, also der heutige Kamerablick (LH-36). Er wird
  damit ein Sonderfall der 3D-Ansicht und kein eigener Code mehr.

Dazu zwei Umkehrungen, die die Werkzeuge brauchen:

- `floor_at(eye, pixel) -> Option<Point>`: Strahl durch das Pixel, Schnitt mit dem Boden.
  Liefert dieselben Plan-Meter, die die Werkzeuge heute vom Plan bekommen.
- `height_at(eye, pixel, anchor) -> f64`: Schnitt mit der senkrechten Ebene durch den
  Griffpunkt, zur Kamera gedreht (Höhe ziehen).

## 5. Renderer: eigener Rasterer mit Tiefenpuffer

### 5.1 Ablauf

```
  Szene (Körper, Flächen)
     │  Flächen → Dreiecke (konvex: Fächer; Grundriss-Polygon: Ohrenschneiden)
     ▼
  Dreiecke in Kamera-Koordinaten
     │  an der Nahebene schneiden (Kamera steht im Raum), dann projizieren
     ▼
  Rasterer: je Pixel Tiefe, Rolle, Helligkeit, Quelle          ◄── kein RGB
     │  Kanten: wo Quelle oder Normale zum Nachbarpixel wechselt, Kantenbit setzen
     ▼
  Frame { width, height, role: [u8], shade: [u8], edge: [bit], source: [u32], depth: [f32] }
     │
     ├─► app/: Rolle → Kante-Farbe × Helligkeit, Kante → Linienfarbe  ──► QImage ──► QML
     ├─► Klick: source[pixel]  (genau, was man sieht)
     └─► Beschriftungen: Ankerpunkt projiziert, verdeckt wenn depth davor
```

- **Farben bleiben Rollen.** Der Rasterer schreibt Rolle und Helligkeit, keine Farbe. Erst
  `app/` färbt mit Kante, `export/` mit der Kante-Palette. Das hält die Regel aus
  `ARCHITECTURE.md` („colours are roles“) und macht hell/dunkel ohne Neuberechnung möglich.
- **Kanten aus dem Puffer statt als Linien:** Umrisse und Knicke entstehen, wo Quelle oder
  Normale zwischen Nachbarpixeln wechseln. Kein Linienrasterer, kein Flimmern von Linien auf
  Flächen, und die Kanten sind von selbst verdeckt.
- **Licht:** feste Richtung plus Grundhelligkeit, je Fläche ein Wert (flach schattiert).
  Reicht, um Seiten von Deckeln zu unterscheiden.
- **Fehlende Höhe:** Raster-Durchsichtigkeit (jedes zweite Pixel im Schachbrett), damit der
  Tiefenpuffer ohne Sortierung auskommt; dazu Kantenrolle „fehlt“ (orange, gestrichelt
  wirkt durch das Muster).
- **Schnitt:** Wände in der Orbit-Ansicht auf 1,20 m gekappt (Schnittebene beim Schneiden
  der Dreiecke), nur Ansicht, nicht Modell. Ein Druck auf C schaltet ihn.
- **Auswahl:** eigene Kantenrolle (Kante „hl“), Fläche unverändert.

### 5.2 Leistung

Rechnung: ein Set hat einige hundert bis wenige tausend Dreiecke; Kosten bestimmen die Pixel.

| Fall | Pixel | Ziel |
|---|---|---|
| Kamerablick in der Shot-Karte | ~0,2 Mio. | < 5 ms |
| 3D-Ansicht, 1920 × 1080 | ~2 Mio. | < 16 ms |
| dieselbe beim Drehen und Ziehen | halbe Auflösung | < 8 ms |

- Gerechnet wird nur bei Änderung (Modell, Kamera, Größe, Abspielkopf), nicht in jedem Takt.
- Reicht ein Kern nicht, werden die Zeilen in Bänder geteilt (`std::thread::scope`), ohne
  Abhängigkeit.
- `f32` im Rasterer, `f64` im Modell.
- Ein Leistungstest im Release-Build hält das Ziel (Regel: Leistung ist ein Fehler mit Test).

### 5.3 Anzeige in QML

Ein eigenes `QQuickPaintedItem` über cxx-qt (vorgesehen schon in `KONZEPT.md`, Zeile
„Plan-Zeichenfläche“), das ein `QImage` hält und zeichnet. Die Brücke füllt das Bild aus dem
Frame. Kein Bild-Provider (URL-Neuladen, Cache), kein Canvas (zu langsam für Pixel).

### 5.4 Druck

Der Export braucht Vektoren. Dieselbe Szene liefert Flächen, nach Tiefe sortiert
(heutiges Verfahren aus `sight.rs`, verbessert: Flächen statt Körper sortieren, Wände an
Öffnungen geteilt). Kleine Fehler bei sich durchdringenden Körpern sind auf Papier
hinnehmbar. PNG-Export nimmt direkt den Rasterer.

## 6. Bedienung

| Was | Plan | 3D (Orbit) |
|---|---|---|
| Ansicht wechseln | Tab | Tab |
| Werkzeuge (Wand, Raum, Tür, Fenster, Objekt, Figur, Kamera, Licht) | Klick auf Plan-Punkt | Klick, Strahl auf den Boden → derselbe Plan-Punkt |
| Auswählen, Verschieben | wie heute | Klick trifft `source[pixel]`, Ziehen auf dem Boden |
| Höhe ziehen (P) | Maus nach oben, Wert am Mauszeiger | am Objekt nach oben ziehen |
| Maßfeld | Länge beim Zeichnen, `5;4` für einen Raum, Zahl = Höhe der Auswahl | gleich |
| Höhen ergänzen (H) | springt zur nächsten freien Form ohne Höhe, Wert tippen, Enter | gleich, Ansicht folgt dem Objekt |
| Navigation | rechte Taste schieben, Rad zoomen | rechte Taste oder Alt + linke drehen, mittlere / Umschalt+rechte schieben, Rad zoomt zum Mauszeiger |
| Durch Kamera sehen | – | Shot-Kamera als `Eye::Shot` |

- Im Plan zeigt jedes Objekt seine Höhe als kleine Zahl; eine fehlende als „Höhe ?“ in der
  Warnrolle. Die Kopfleiste zeigt „n ohne Höhe · ergänzen (H)“.
- Die Werkzeuge in `app/tools.rs` bekommen weiter Plan-Meter. Neu ist nur, woher: im Plan
  rechnet QML die Pixel um wie heute, in 3D fragt die Brücke `floor_at` über `services/`.
- Raum-Werkzeug (Rechteck → vier Wände) und freie Form sind neue Werkzeuge im Modus Bauen.

## 7. Bausteine und Schichten

| Teil | Ort | Begründung |
|---|---|---|
| Höhen, Szene, `Eye`, `floor_at`, `height_at` | `stellprobe/` (Module `height`, `scene`, `eye`) | Geometrie und Berechnung gehören nach `stellprobe/` |
| Rasterer (Dreiecke → Frame) | **neu: `raster/`**, hängt von nichts ab | allgemein, ohne Wissen über Sets; allein testbar; hält `stellprobe/` unter den Grenzen der Strukturprüfung |
| 3D-Ansicht, Kamerablick | `services/` (Sitzung hält `Eye`, liefert Frame und Beschriftungen) | eine Tür für alle Oberflächen |
| Färben, `QQuickPaintedItem`, Eingaben | `app/` | Oberfläche |
| Vektor-Kamerablick, PNG | `export/` | wie heute |

`raster/` liegt wie `imagehead/` neben den anderen und hängt von nichts ab; `stellprobe/`
benutzt es. Das ändert „Layers“ in `ARCHITECTURE.md` und braucht die Bestätigung des
Eigentümers; erteilt am 15.11.2026 (Abschnitt 9).

## 8. Schritte (je ein PR, jeder für sich nützlich)

1. **Höhen im Modell:** Raumhöhe, Wand-, Brüstungs-, Sturzhöhe, Unterkante, `height`-Modul;
   Felder in den Karten; der Kamerablick nimmt die echten Höhen. Noch kein neues 3D.
2. **Rasterer und Szene:** `raster/` mit Tests als ASCII-Bild (kleiner Frame, Rollen als
   Zeichen, im Test lesbar); Szene aus dem Set; Kamerablick in der Shot-Karte zeigt das
   Rasterbild über `QQuickPaintedItem`. Verdeckung ist ab hier richtig.
3. **3D-Ansicht:** `Eye::Orbit`, Tab, Navigation, Klicks per `floor_at` und `source`,
   Auswählen, Verschieben, Setzen von Objekten, Figuren, Kameras in 3D; Beschriftungen.
4. **Höhe ziehen, Maßfeld, Höhen ergänzen;** Höhen im Plan sichtbar.
5. **Freie Form, Raum-Werkzeug, Bauformen der Bibliothek** (Daten in `store/library/`).
6. **Schnitt, Blick durch die Kamera, Export** (PNG aus dem Rasterer, Vektor sortiert).

Neue Anforderungen dafür (Vorschlag, Nummern vergibt der Eigentümer): 3D-Arbeitsansicht mit
denselben Werkzeugen; Raumhöhe und Höhen von Wand und Öffnung; freie Formen mit Höhe;
Höhen ergänzen; Bauformen der Bibliothek.

## 9. Entscheidungen des Eigentümers (15.11.2026)

1. Rasterer als eigener Baustein `raster/`, hängt von nichts ab, `stellprobe/` benutzt ihn.
   „Layers“ in `ARCHITECTURE.md` ändert der PR, der ihn anlegt (Schritt 2).
2. Freie Form ist ein Objekt: `ObjectShape::Polygon`, Ebene „Set“, Karte wie jedes Objekt.
3. Der Kamerablick in der Shot-Karte wird ein Rasterbild; Vektor bleibt für den Druck.
4. Körpergröße je Figur: später.
5. Drehen in 3D mit rechter Taste und mit Alt + linker Taste.

# Prototyp: Plan und 3D auf einem Modell

Wegwerf-Prototyp (Browser, three.js), keine Vorlage für Code. Umsetzung: [`../3D-KONZEPT.md`](../3D-KONZEPT.md). Öffnen: `raumskizze.html`
über einen lokalen Webserver (lädt three.js 0.160 von jsDelivr).

## Idee

```
 Element = Grundriss (x, y)  +  Sockel base_m  +  Höhe height_m (leer = Vorgabe)
              │                         │
              ▼                         ▼
        Plan: Umriss              3D: Extrusion
```

- Plan und 3D sind zwei Kameras auf dieselbe Szene, kein zweiter Modus.
- Werkzeuge treffen per Strahl den Boden (Plan-Koordinaten); darum gleich in beiden
  Ansichten. Nur „Höhe ziehen“ braucht die dritte Achse (3D: Strahl, Plan: Maus nach oben).
- Höhe entsteht beim Zeichnen: Wand ← Raumhöhe, Bibliotheksobjekt ← Bibliothek,
  Figur ← Haltung, Öffnung ← Brüstung/Sturz der Art. Nur freie Formen starten ohne Höhe;
  „ergänzen (H)“ springt durch sie, Höhe tippen, Enter, nächste.
- Sockel automatisch: ein Objekt oder eine stehende Figur steht auf dem größten Objekt
  unter seiner Mitte (Figur auf Podest).
- Maßfeld wie SketchUp/BenchUp: Länge tippen beim Wandzeichnen, `5;4` für einen Raum,
  Zahl für die Höhe der Auswahl.
- 3D-Schnitt: Wände in der Ansicht auf 1,20 m gekappt, damit man in den Raum sieht.

## Übertragung auf Klappe

- `model/`: `Wall.height_m: Option<f64>` und `Set.wall_height_m`; `Opening.sill_m`,
  `head_m`; `SetObject.base_m: Option<f64>`; `ObjectShape::Polygon { points }` für freie
  Formen. `SetObject.height_m` und die Bibliothekshöhen gibt es schon (LH-17).
- `stellprobe/`: Höhenauflösung (Vorgabe, Sockel, Stapeln) an einer Stelle; `sight.rs` und
  der Kamerablick nehmen sie statt `WALL_HEIGHT_M`/`OBJECT_HEIGHT_M`. Eine 3D-Arbeitsansicht
  ist der Kamerablick mit frei geführter Kamera; Treffer per Strahl auf den Boden ergeben
  dieselben Plan-Punkte, die die Werkzeuge heute bekommen.
- Darstellung: Qt Quick 3D ist GPLv3 (`KONZEPT.md`); die eigene Projektion reicht für
  Quader, braucht für eine bedienbare 3D-Ansicht aber Tiefensortierung oder einen
  Tiefenpuffer. Offene Entscheidung des Eigentümers.
- BenchUp (MIT, Rust) ist ein Möbel-CAD, kein Set-Werkzeug; übertragbar sind Push/Pull,
  Maßeingabe und „Draufsicht ist nur eine Kamera“, kein Code.

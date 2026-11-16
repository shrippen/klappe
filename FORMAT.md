# Projektformat

Ein Klappe-Projekt ist ein Ordner aus Textdateien, damit Git es versionieren und vergleichen
kann (NF-03). Umsetzung: `crates/store`; Beispiel: `testdata/beispiel.klappe/`.

```
<name>.klappe/
├── projekt.toml         Formatversion, Titel
├── figuren.toml         Figuren, je ein [[figur]]
├── bibliothek.toml      eigene Objekte der Bibliothek, je ein [[objekt]] (optional)
├── vorlagen.toml        eigene Vorlagen, je ein [[vorlage]] (optional)
├── drehorte/<id>.toml   ein Drehort samt Set (Wände, Objekte)
├── szenen/<id>.json     eine Szene: Nummer, Überschrift, Drehort, Floorplans
├── bilder/<name>.png    Bilder für Hintergründe (PNG, JPEG), vom Nutzer hinzugefügt
├── drehbuch.fountain    Drehbuch als Text (optional)
└── .sicherung/          Sicherung ungespeicherter Änderungen (optional, siehe unten)
```

## Regeln

- **Formatversion:** `format_version` in `projekt.toml` (aktuell 2). Eine neuere Version als die
  des Programms wird nicht geladen, sondern gemeldet.
- **Papier:** `paper` in `projekt.toml`, z. B. `"a4-portrait"`: Größe (`a4`, `a3`, `letter`),
  Bindestrich, Ausrichtung (`portrait`, `landscape`). Das Blatt, auf dem zuletzt ausgegeben wurde;
  der Ausgabedialog beginnt damit. Fehlt der Schlüssel (und er wird bei `a4-landscape` nicht
  geschrieben), gilt A4 quer; ein unbekannter Teil gilt als sein Standard (`a3` ist A3 quer,
  `tabloid` A4 quer), ein Wert, der kein Text ist, lädt nicht. Keine neue Formatversion: ältere
  Fassungen verlieren ihn beim Speichern.
- **Ids:** Drehort- und Szenen-Ids sind Dateinamen und müssen mit der `id` in der Datei
  übereinstimmen. Nicht erlaubt: leer, `/`, `\`, `..`, Beginn mit `.`, Ende auf `.` oder Leerzeichen,
  `<>:"|?*`, Steuerzeichen, Windows-Gerätenamen (`con`, `nul`, `com1` …). Ids, die sich nur
  in der Groß-/Kleinschreibung unterscheiden, gelten als gleich.
- **Reihenfolge:** Drehorte und Szenen werden nach Id sortiert gelesen. Die Reihenfolge der
  Figuren steht in `figuren.toml`, die der Shots (Schnittfolge) im Floorplan.
- **Shots:** `size` (Einstellungsgröße), `angle` (Perspektive) und `movement` sind Namen in
  Kleinschreibung mit Bindestrich. `angle`: `eye-level` (Normalsicht), `high` (Aufsicht), `low`
  (Untersicht), `birds-eye` (Vogelperspektive), `worms-eye` (Froschperspektive), `dutch`
  (gekippt). Fehlt `angle`, gilt `eye-level`. Die Drehreihenfolge (Setups gruppiert) steht nicht
  in der Datei; sie ergibt sich aus Kamera, Ort und Objektiv. Ausnahme `setup_of` (LH-77, optional, nur geschrieben,
  wenn gesetzt): die `id` eines anderen Shots desselben Floorplans; der Shot wird im Setup dieses
  Shots gedreht. Die Shots ohne `setup_of` bilden die Setups (nach erstem Shot nummeriert); ein
  Shot mit `setup_of` zieht keinen anderen mit und ändert keine Nummer. Ziel darf kein Shot mit
  `setup_of` sein (keine Ketten) und nicht der Shot selbst. Fehlt das Ziel oder ist es selbst
  zugeordnet, wird der Shot wie ohne `setup_of` berechnet und die Oberfläche weist darauf hin.
  Fehlt `setup_of`, gilt die Berechnung.
- **Achse (LH-40):** `axis` im Floorplan (`szenen/<id>.json`) mit `a` und `b`, den Figur-Ids,
  zwischen denen die 180°-Achse liegt; je Floorplan, weil die Spuren der Figuren dort liegen.
  Fehlt `axis`, gilt die Achse zwischen den ersten zwei Figuren des Floorplans. Wird eine der
  beiden Figuren entfernt, entfällt `axis`.
- **JSON:** eingerückt, Schlüssel in der Feldreihenfolge des Modells, Zeilenumbruch am Ende.
- **Einheiten:** im Feldnamen (`_m`, `_deg`, `_s`, `_mm`, `_k`, `_w`); Ursprung oben links, y nach
  unten.
- **Speichern:** jede Datei wird in eine Temp-Datei (`.<name>.tmp`) im selben Ordner geschrieben
  und dann umbenannt; ein Absturz hinterlässt nie eine halbe Datei (LH-05). Dateien in
  `drehorte/` und `szenen/`, deren Id nicht mehr zum Projekt gehört, werden gelöscht, sonst
  nichts. Ohne Drehbuchtext bleibt eine vorhandene `drehbuch.fountain` unberührt.
- **Hintergrund:** `[set.background]` im Drehort (Datei relativ zum Projektordner, linke obere
  Ecke `at`, `width_m`, `rotation_deg` im Uhrzeigersinn um `at`, `opacity`). Die Höhe folgt aus
  dem Seitenverhältnis des Bildes. Fehlt der Abschnitt, gibt es keinen Hintergrund.
- **Objekte und Bibliothek (LH-11, LH-12, LH-25):** `[[set.objects]]` im Drehort haben `kind`,
  `label`, Mitte `at`, `width_m`, `depth_m` und `rotation_deg`; dazu `shape`: `rect` oder
  `ellipse` (in `width_m` mal `depth_m` einbeschrieben). Fehlt `shape`, gilt `rect`; ein Rechteck
  schreibt den Schlüssel nicht, ältere Projekte laden und speichern unverändert. Ein `kind`,
  den keine Bibliothek kennt, bleibt ladbar und wird als Rechteck in der eigenen Größe
  gezeichnet. Die eingebaute Bibliothek ist Teil des Programms (`crates/store/library/standard.toml`,
  Namen im Katalog unter `objectKind.<kind>`). `bibliothek.toml` hält die eigenen Objekte des
  Projekts, je ein `[[objekt]]` mit `kind` (eindeutig, auch gegenüber den eingebauten), `label`
  (Name), `width_m`, `depth_m` und `shape`. Dieselbe Struktur hat eine Bibliotheksdatei zum
  Weitergeben (beliebiger Name, `*.toml`). Optional `group` (LH-12): die Gruppe, unter der die
  Auswahlliste den Eintrag zeigt; in der eingebauten Bibliothek ein Schlüssel (`furniture`,
  `build`, `vehicles`, `nature`, `camera`, `crew`, `props`; Namen im Katalog unter
  `objectGroup.<group>`), bei
  eigenen Objekten freier Text. Ohne Gruppe steht ein eigenes Objekt unter „Eigene“; der Schlüssel
  fehlt dann in der Datei. Ein eigener Eintrag lässt sich umbenennen (`label`, `group`) und
  löschen; Objekte im Plan behalten `kind`, Maße, Form und Namen, ihr `kind` ist dann wie ein
  unbekanntes. Eine Datei mit einem unbrauchbaren Eintrag (leerer
  `kind`, Maß nicht in [0,05, 1000] m, doppelter `kind`) wird als Ganzes abgelehnt; die
  `bibliothek.toml` eines Projekts dann beim Öffnen mit Meldung. Ohne eigene Objekte gibt es die
  Datei nicht: `save` löscht sie dann. Keine neue Formatversion: ältere Fassungen kennen `shape`,
  `group` und `bibliothek.toml` nicht und verlieren sie beim Speichern.
- **Bauform (LH-97):** ein Eintrag der Bibliothek (eingebaut oder eigen) hat optional `form`, den
  Namen einer eingebauten Bauform; fehlt der Schlüssel, bleibt das Objekt in 3D ein Prisma seines
  Umrisses. Die Bauformen sind Daten (`crates/store/library/forms.toml`): je `[[form]]` ein `name`
  und je `[[form.box]]` ein Quader `from = [x, y, z]`, `to = [x, y, z]` in Anteilen (0 bis 1) des
  Objekts: x entlang der Breite, y entlang der Tiefe (von der Ecke oben links, vor dem Drehen),
  z von der Unterkante bis zur Oberkante (die Höhe des Objekts). Je Achse `from` kleiner als `to`;
  der Name ist eindeutig. Eine Bibliotheksdatei, deren Eintrag eine unbekannte `form` nennt, wird
  als Ganzes abgelehnt. Welche Bauform ein Objekt im Plan bekommt, folgt aus seinem `kind`
  (eigener Eintrag zuerst, dann eingebaut); `[[set.objects]]` haben kein Feld dafür. Nur
  Rechtecke bekommen eine Bauform, keine Ellipse und keine freie Form. Keine neue
  Formatversion: ältere Fassungen verlieren `form` beim Speichern.
- **Spiegeln (LH-115):** `[[set.objects]]` haben optional `mirrored` (`true`: links und rechts
  vertauscht, entlang der eigenen Breite, vor dem Drehen). Fehlt der Schlüssel, gilt `false`;
  ein nicht gespiegeltes Objekt schreibt ihn nicht, ältere Projekte laden und speichern
  unverändert. Die Bauform in 3D (LH-97) wird gespiegelt; ein Rechteck und eine Ellipse sehen im
  Plan gleich aus. Eine freie Form (`polygon`) hat das Feld nicht nötig: Spiegeln kehrt ihre
  Punkte um (x wird -x, Reihenfolge umgekehrt), `mirrored` bleibt `false`. Keine neue
  Formatversion (noch kein Release), ältere Fassungen verlieren das Feld beim Speichern.
- **Art ändern, Maße zurücksetzen (LH-114, LH-116):** keine Felder, nur Bearbeitungen. Eine neue
  Art (`kind`) behält Mitte, Drehung, Ebene und Unterkante. Breite, Tiefe, Höhe und Name folgen
  dem neuen Bibliothekseintrag, solange das Objekt noch die des alten Eintrags hat; hat der
  Nutzer sie geändert (oder kennt keine Bibliothek die alte Art), bleiben sie. Die Form
  (`shape`) folgt dem neuen Eintrag, außer bei einer freien Form. Eine Art ohne Eintrag und die
  freie Form (`form`) als Ziel werden abgelehnt. „Maße zurücksetzen“ setzt Breite, Tiefe und
  Höhe auf die des Eintrags der Art; ohne Eintrag wird es abgelehnt.
- **Höhe von Objekten (LH-17):** `[[set.objects]]` und `[[objekt]]` der Bibliothek haben optional
  `height_m`, die Höhe in Metern (mindestens 0,05, höchstens 100). Fehlt der Schlüssel, ist keine
  Höhe angegeben (Fläche, Teppich); ein Objekt ohne Höhe schreibt ihn nicht, ältere Projekte
  laden und speichern unverändert. Ein neues Objekt übernimmt die Höhe seines
  Bibliothekseintrags, „Als Bibliotheksobjekt ablegen“ die des Objekts. Eine Bibliotheksdatei mit
  unbrauchbarer Höhe wird als Ganzes abgelehnt. Keine neue Formatversion, wie bei den Notizen.
- **Höhen des Sets (LH-90, LH-91), seit Formatversion 2:** `[set]` hat `wall_height_m`, die
  Raumhöhe (Standard 2,6); fehlt der Schlüssel, gilt 2,6. Jede `[[set.walls]]` hat optional
  `height_m` (sonst Raumhöhe), jede Öffnung optional `sill_m` (Brüstung) und `head_m` (Sturz),
  sonst die ihrer Art: Tür 0 und 2,1, Fenster 0,9 und 2,1. Jedes `[[set.objects]]` hat optional
  `base_m`, die Unterkante; fehlt sie, steht das Objekt auf dem größten Objekt mit Höhe, das
  größer ist und seine Mitte trägt, sonst auf dem Boden. Ein Objekt ohne `height_m` wird in
  der Ansicht 0,9 m hoch gezeigt und trägt nichts. Leere Werte schreibt `save` nicht.
  Raum-, Wand- und Sturzhöhe liegen in [0,05, 100] m, Brüstung und Unterkante in [0, 100] m, die
  Brüstung unter dem Sturz; sonst wird der Drehort beim Öffnen abgelehnt.
  Formatversion 1 lädt unverändert (Raumhöhe 2,6, die feste Wandhöhe von früher) und wird als
  2 gespeichert; eine Fassung für Version 1 lehnt das Projekt danach ab.
- **Freie Form (LH-92):** ein Objekt mit `shape` `polygon`: ein Umriss aus Punkten statt Rechteck
  oder Ellipse. Im Drehort steht es als Tabelle `[set.objects.shape.polygon]` mit der Liste
  `[[set.objects.shape.polygon.points]]`, je Punkt `x_m` und `y_m`. Die Punkte liegen in Metern
  relativ zur Mitte `at` des Objekts, vor der Drehung `rotation_deg`, in der Reihenfolge um den
  Umriss. `width_m` und `depth_m` sind die Größe des Kastens um die Punkte; die Mitte des Kastens
  ist `at`. Ändert man die Größe, wird der Umriss auf sie gestreckt, die Punkte bleiben
  unverändert gespeichert. Eine freie Form ist ein Objekt wie jedes andere (Ebene Set, `height_m`,
  `base_m`; eine neue hat `kind = "form"` und keine Höhe). Eine Bibliotheksdatei und
  `bibliothek.toml` nehmen dieselbe `shape` an. Laden lehnt einen Umriss mit weniger als drei
  Punkten, einer Zahl, die keine ist, ohne Fläche, mit einem Kasten unter 0,05 m in einer Richtung
  oder größer als 1000 m ab (Drehort mit Meldung,
  Bibliotheksdatei als Ganzes). Keine neue Formatversion (noch kein Release).
- **Kamerahöhe und Neigung (LH-34):** jeder Kamera-Mark (`marks` in `szenen/<id>.json`) hat
  `height_m` (Höhe des Objektivs über dem Boden, 0 bis 50) und `tilt_deg` (Standard 0, −90 bis
  90). Die Neigung gilt vom Horizont aus: positiv blickt nach oben, negativ nach unten, 0 ist
  waagerecht. Beides gab es schon im Format; neu sind die Grenzen und die Bedienung. Keine
  neue Formatversion.
- **Gerät und Brennweite je Kamera-Mark (LH-33):** jeder Kamera-Mark (`marks` in
  `szenen/<id>.json`) hat optional `rig`, das Gerät, mit dem die Kamera vom vorigen Mark hierher
  kommt: `tripod` (Stativ: nur Schwenk und Neigung, der Ort bleibt der des vorigen Marks),
  `dolly` (Standard), `track` (Schiene), `crane`, `handheld`, `gimbal` (Steadicam oder Gimbal),
  und optional `lens_mm`, die Brennweite an diesem Mark (größer als 0, höchstens 5000). Am ersten
  Mark zählt `rig` nicht. Zwischen zwei Marks mit `lens_mm` läuft die Brennweite linear mit der
  Zeit (nach der Pause des ersten bis zur Ankunft am zweiten, wie der Ort); davor und danach
  hält sie. Höhe und Neigung (`height_m`, `tilt_deg`) laufen ebenso linear zwischen den Marks.
  Hat irgendein Mark der Kamera `lens_mm`, gilt allein das; sonst gelten wie bisher
  das Objektiv des Shots, dann das der Kamera. Ein Wert mit Standard (`dolly`, keine Brennweite)
  wird nicht geschrieben; ältere Projekte laden und speichern unverändert. Ein Mark mit
  `tripod` an einem anderen Ort als der vorige lässt die Bearbeitung nicht zu; eine Datei mit
  einem solchen Mark lädt, der Ort gilt. Keine neue Formatversion, wie bei den Notizen.
- **Notizen (LH-16):** `[[set.notes]]` im Drehort: `id` (`n1`, `n2` …, innerhalb des Drehorts),
  Ort `at` in Metern, `text` und die Breite `width_m` des Kärtchens in Metern (mindestens 0,6,
  schmaler lädt nicht; fehlt sie, gilt 2,4). Fehlt der Abschnitt, gibt es keine Notizen; ohne Notizen wird
  er nicht geschrieben, ältere Projekte laden und speichern unverändert. Keine neue
  Formatversion: ein Feld mehr, das ältere Fassungen nicht kennen und beim Speichern verlieren.
- **Beschriftung an ein Element gebunden (LH-118):** eine Notiz (`[[set.notes]]`) hat optional
  `bound`, den Schlüssel des Elements, dessen Beschriftung sie ist: `object:<id>` (Objekt des
  Drehorts), `figure:<plan>:<figur>` (die Figur am Abspielkopf im Plan `<plan>`) oder
  `mark:<plan>:<figur>:<index>` (der Mark mit diesem Index, von 0, im Plan `<plan>`). Figur und
  Mark gehören zu einem Plan, weil die Notizen dem Drehort gehören und jeder Plan eigene Spuren
  hat; Bearbeitungen in einem anderen Plan lassen die Notiz stehen. `at` bleibt der Ort des
  Kärtchens. Die Bearbeitung verschiebt es um dieselbe Strecke wie das Element; zieht man die
  Notiz selbst, bleibt sie gebunden und das Abstandsmaß ändert sich. Fehlt das Element
  (entfernt) oder ändert sich die Zahl der Marks der Figur (nur bei `mark:`, deren Index sonst
  einen anderen Mark meint), wird die Bindung gelöst, die Notiz bleibt stehen. Eine Kopie der
  Notiz (Strg+D) ist ungebunden. Bei gesperrter Ebene Notizen lehnt die Sitzung eine
  Bearbeitung ab, die eine gebundene Notiz verschieben oder lösen würde. Ohne Bindung fehlt der
  Schlüssel; ein unlesbarer Schlüssel bindet nichts. Ältere Projekte laden und speichern
  unverändert, ältere Fassungen verlieren `bound` beim Speichern. Keine neue Formatversion.
- **Bilder (LH-18):** `[[set.images]]` im Drehort: Fotos und Skizzen, frei auf dem Plan. Je Bild
  `id` (`i1`, `i2` …, innerhalb des Drehorts), `file` (relativ zum Projektordner, die Datei liegt
  in `bilder/`, PNG oder JPEG wie beim Hintergrund), die Mitte `at` in Metern, die Breite
  `width_m` in Metern (zwischen 0,05 und 1000, sonst lädt der Drehort nicht) und `rotation_deg`
  im Uhrzeigersinn um die Mitte (0 wird nicht geschrieben) und `opacity` (Deckkraft, 0,1 bis 1;
  fehlt sie, gilt 1, und 1 wird nicht geschrieben; ein Wert außerhalb lädt nicht). Die Höhe folgt aus dem
  Seitenverhältnis des Bildes. Ein Bild liegt auf der Ebene der Notizen. Fehlt die Datei, zeigt
  der Plan einen leeren Rahmen (4:3) mit dem Dateinamen; das Bild bleibt im Projekt. Fehlt der
  Abschnitt, gibt es keine Bilder; ohne Bilder wird er nicht geschrieben. Keine neue
  Formatversion (noch kein Release), ältere Fassungen verlieren die Bilder beim Speichern.
- **Türen und Fenster in Wänden (LH-10):** jede `[[set.walls]]` kann `[[set.walls.openings]]`
  tragen: `kind` (`door` oder `window`), `at_m` (Meter von `from` der Wand bis zur Mitte der
  Öffnung, entlang der Wand) und `width_m` (mindestens 0,30 m, höchstens 1000). Die Wand hat dort eine Lücke; die Tür
  schwingt auf dem Plan standardmäßig nach rechts, von `from` Richtung `to` gesehen, das
  Scharnier liegt am Anfang der Lücke (der Seite von `from`). Optional `side` (`right` oder
  `left`, Standard `right`) und `hinge` (`start` oder `end`, Standard `start`) ändern das;
  beide fehlen in der Datei, wenn sie dem Standard entsprechen, ein anderer Wert lässt das
  Laden scheitern, ein Fenster ignoriert sie. Ohne Öffnungen steht der Schlüssel nicht in der Datei, ältere Projekte
  laden und speichern unverändert. Laden lehnt eine Öffnung ab, deren Breite oder Lage keine
  Zahl ist oder deren Breite unter 0,30 m liegt. Eine Öffnung, die über das Wandende reicht
  oder eine andere überdeckt, lädt; die Bearbeitung lässt so etwas nicht zu. Öffnungen werden
  über Wand und Index angesprochen, nicht über eine Id. Treppen bleiben Objekte der Bibliothek
  (`kind = "stairs"`); `door` und `window` gibt es dort weiter als Objekte ohne Wand. Keine
  neue Formatversion, wie bei den Notizen.
- **Zeichnungen (LH-19):** `drawings` im Floorplan der Szene (`szenen/<id>.json`), je Eintrag
  `id` (`d1`, `d2` …, innerhalb des Floorplans), `shape`, `color`, `dash` und `fill`. `shape` hat
  `kind`: `line` und `arrow` (Spitze bei `to`) mit `from` und `to`; `rect` und `ellipse` mit zwei
  gegenüberliegenden Ecken `from` und `to` (achsenparallel); `area` mit `points` (geschlossenes
  Polygon, mindestens drei). Punkte in Metern. `color` ist eine Rolle wie bei Figuren (`purple`,
  `blue`, `green`, `yellow`, `orange`, `red`, `cyan`), `dash` `solid`, `dashed` oder `dotted`
  (Standard `solid`), `fill` `outline` oder `filled` (Standard `outline`; nur Rechteck, Ellipse und
  Fläche haben ein Inneres). Optional `rotation_deg` (Grad, im Uhrzeigersinn, Standard 0, bei 0 nicht
  geschrieben): die Zeichnung ist um die Mitte des Kastens ihrer Punkte gedreht, ein gedrehtes
  Rechteck oder eine gedrehte Ellipse ist also nicht mehr achsenparallel; die Punkte bleiben ungedreht
  gespeichert. Ohne Zeichnungen fehlt der Schlüssel; ältere Projekte laden und
  speichern unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Lampen (LH-50, LH-51):** `lights` im Floorplan der Szene (`szenen/<id>.json`), je Eintrag
  `id` (`l1`, `l2` …), `label`, `kind` (`fresnel`, `led-panel`, `tube`, `balloon`, `reflector`,
  `flag`), `at` in Metern, `facing_deg` (Richtung, 0° nach rechts, im Uhrzeigersinn),
  `beam_deg` (Abstrahlwinkel, Kegel im Plan und im Export), `color_temp_k` und `power_w`. Nur
  LED-Lampen (`led-panel`, `tube`) stellen die Farbtemperatur ein. Die eingebaute
  Lampenbibliothek (`crates/store/light/standard.toml`) steht nicht im Projektordner: eine Lampe
  aus ihr übernimmt Art, Winkel, Leistung und Farbtemperatur als gewöhnliche Werte, ohne Verweis
  auf den Eintrag. Keine neue Formatversion; die Felder gab es schon.
- **Kurvenwege (LH-21):** jeder Mark einer Figur und einer Kamera (`marks` in `szenen/<id>.json`)
  hat optional `via`, eine Liste von Punkten in Metern (Knicke): der Weg vom vorigen Mark zu
  diesem ist die glatte Kurve (Catmull-Rom-Spline) durch den vorigen Mark, die Punkte von `via`
  in dieser Reihenfolge und diesen Mark; sie geht durch jeden Punkt. Ohne `via` ist der Weg
  gerade; am ersten Mark zählt `via` nicht. Die Bewegung läuft entlang der Kurve mit
  gleichmäßigem Tempo, die Ankunftszeiten bleiben die der Marks. Ohne Knicke fehlt der
  Schlüssel; eine leere Liste wird nicht geschrieben. Bruch vor dem ersten Release, ohne
  Übergang: `via` war ein einzelner Punkt (Kontrollpunkt einer quadratischen Bézierkurve);
  Beispielprojekte, Demo und Testdaten sind umgestellt, der Punkt der Liste ist die Mitte der
  alten Kurve, (a + 2 via + b) / 4 mit a, b als Orten des vorigen und dieses Marks.
- **Auftritt und Abgang (LH-26):** jeder Mark einer Figur (`marks` in `szenen/<id>.json`) hat
  optional `arrival` (`walk` oder `appear`, Standard `walk`) und `departure` (`stay` oder
  `leave`, Standard `stay`). Bei `appear` ist die Figur vor `at_s` dieses Marks nicht zu sehen
  und erscheint dort, statt vom vorigen Mark zu laufen; am ersten Mark heißt das: vor `at_s`
  unsichtbar. Bei `leave` ist sie nach `at_s + hold_s` nicht mehr zu sehen, bis ein späterer Mark
  `appear` hat. Ein Wert mit Standard wird nicht geschrieben; ältere Projekte laden und speichern
  unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Gruppen (LH-23):** eine Figur in `figuren.toml` hat optional `group`, die Zahl der Personen
  (2 bis 999). Mit `group` ist die Figur eine Gruppe (Gäste, Wachen): sie hat eine Spur, Marks
  und Wege wie jede Figur und wird größer gezeichnet, mit drei Punkten im Umriss, ihr Name trägt
  die Zahl (`Gäste (12)`). Ohne Gruppe fehlt der Schlüssel; ältere Projekte laden und speichern
  unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Größe von Figuren (LH-25):** eine Figur in `figuren.toml` hat optional `size_m`, den
  Durchmesser des Körpers in Metern (0,1 bis 20), etwa Kind, Pferd oder Kutsche. Ohne `size_m`
  gilt der Standard (0,7 m). Eine Gruppe bleibt um denselben Faktor größer als eine einzelne
  Figur. Ohne Größe fehlt der Schlüssel; ältere Projekte laden und speichern unverändert.
  Laden lehnt einen Wert außerhalb 0,1 bis 20 ab. Keine neue Formatversion, wie bei den
  Gruppen.
- **Vorlagen (LH-07):** eine Vorlage ist ein Satz Figuren-Platzhalter und Kameras mit Marks für
  eine typische Auflösung. Die eingebauten sind Teil des Programms
  (`crates/store/template/standard.toml`: `shot-reverse`, `triangle`, `long-take`; Namen im Katalog
  unter `template.<id>`). `vorlagen.toml` hält die eigenen des Projekts, je ein `[[vorlage]]` mit
  `id` (eindeutig im Projekt), `label` (Name) und beliebig vielen `[[vorlage.figures]]` und
  `[[vorlage.cameras]]`. Eine Figur hat `slot` (eindeutig in der Vorlage, auch der Name einer dafür
  angelegten Figur) und `[[vorlage.figures.marks]]` wie ein Mark einer Spur; eine Kamera hat die
  Felder einer Kamera der Szene und `[[vorlage.cameras.marks]]`, `follows` nennt einen `slot`. Orte
  der Marks sind Meter vom Einfügepunkt, Zeiten Sekunden ab Szenenbeginn; `id` und `label` einer
  Kamera sind Platzhalter, eine eingefügte Kamera bekommt freie. Eine Datei mit unbrauchbarer
  Vorlage (leere oder doppelte `id`, leeres `label`, nichts darin, leerer oder doppelter `slot`, Figur oder
  Kamera ohne Marks, `follows` ohne passenden `slot`, eine Zahl `nan` oder `inf`, ein Wert
  außerhalb der Grenzen des Editors) wird beim Öffnen mit Meldung abgelehnt. Die Grenzen
  (`crates/model/src/limits.rs`): Orte bis ±1000 m, Pause nicht negativ, Höhe eines Kamera-Marks
  0 bis 50 m, Neigung bis ±90°, Brennweite 1 bis 5000 mm, Sensorbreite über 0 bis 500 mm,
  Seitenverhältnis über 0 bis 20. Ohne eigene Vorlagen gibt es die Datei nicht: `save` löscht sie dann.
  Keine neue Formatversion, wie bei der Bibliothek.
  Eigene Vorlagen lassen sich umbenennen (`label`) und löschen; die eingebauten nicht. Eine
  Vorlagendatei zum Weitergeben hat dasselbe Format wie `vorlagen.toml` (je `[[vorlage]]`; ohne
  `vorlage` ist sie keine Vorlagendatei) und wird als Ganzes abgelehnt, wenn eine Vorlage
  unbrauchbar ist. Beim Import wird eine Vorlage mit gleichem `label`, gleichen Figuren und
  Kameras übersprungen; ist ihre `id` sonst schon vergeben (eigen oder eingebaut), bekommt sie
  eine freie (`id-2`, `id-3` …).
- **Bilder:** `bilder/` gehört dem Nutzer. `save` löscht dort nie etwas, auch nicht, wenn kein
  Drehort das Bild mehr nutzt. Beim Hinzufügen wird nicht doppelt kopiert (gleicher Name und
  gleicher Inhalt); anderer Inhalt unter demselben Namen bekommt `-2`, `-3` … vor der Endung.
- Fehlende `figuren.toml`, fehlende Ordner und fehlendes Drehbuch gelten als leer.
- **Sicherung (LH-05):** ungespeicherte Änderungen schreibt die Sitzung regelmäßig nach
  `.sicherung/`, ein vollständiger Projektordner im selben Format. Die Projektdateien bleiben
  dabei unberührt; erst der Nutzer speichert, und das Speichern löscht `.sicherung/`, außer die Sitzung
  hat sie beim Öffnen gefunden und der Nutzer hat noch nicht entschieden. Beim
  Öffnen bietet die Sitzung eine abweichende Sicherung zur Wiederherstellung an; eine
  gleiche wird still gelöscht. Ein Sicherungsordner gilt nur mit
  der Datei `fertig`: sie wird zuletzt geschrieben und beim Löschen zuerst entfernt, ein
  Absturz hinterlässt also nie einen lesbaren halben Ordner. Die Sicherung wird als Ganzes
  ersetzt: erst entsteht `.sicherung.neu/` (zuletzt `fertig`), dann wird `.sicherung/` zu
  `.sicherung.alt/` umbenannt, `.sicherung.neu/` zu `.sicherung/` und `.sicherung.alt/`
  gelöscht. Ein fertiger `.sicherung.neu/` ist neuer als `.sicherung/` und wird zuerst
  gelesen; vor dem nächsten Schreiben wird er befördert, nie gelöscht. Ordner ohne `fertig`
  sind Absturzreste und werden ignoriert. Speichert der
  Nutzer nur das Drehbuch, wird die Sicherung entfernt, wenn nur das Drehbuch ungesichert
  war, sonst mit dem gespeicherten Drehbuch neu geschrieben. `load` liest `.sicherung/` nie
  als Teil des Projekts. Git soll die Ordner ignorieren; Vorschlag für `.gitignore` im Projekt:

  ```
  .sicherung/
  .sicherung.neu/
  .sicherung.alt/
  ```

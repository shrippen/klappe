# Projektformat

Ein Klappe-Projekt ist ein Ordner aus Textdateien, damit Git es versionieren und vergleichen
kann (NF-03). Umsetzung: `crates/store`; Beispiel: `testdata/beispiel.klappe/`.

```
<name>.klappe/
├── projekt.toml         Formatversion, Titel, Papier, Drehbuchseiten ([script]), gespeicherte Ansichten
├── figuren.toml         Figuren, je ein [[figur]]
├── orte.toml            Spielorte der Szenenköpfe, je ein [[ort]] (optional)
├── bibliothek.toml      eigene Objekte der Bibliothek, je ein [[objekt]] (optional)
├── vorlagen.toml        eigene Vorlagen, je ein [[vorlage]] (optional)
├── drehorte/<id>.toml   ein Drehort samt Set (Wände, Objekte)
├── szenen.toml          Register der Drehinhalte (Szenen, Content Items), je ein [[szene]], in Projektfolge
├── szenen/<id>.json     Stellprobe je Drehinhalt: Drehort, Zeitpunkt der Sonne, Floorplans (Nummer und Kopf stehen im Register)
├── bilder/<name>.png    Bilder für Hintergründe (PNG, JPEG), vom Nutzer hinzugefügt
├── drehbuch.fountain    Drehbuch als Text (optional)
├── nummern.toml         Sperrstand der Szenennummern (optional, DB-33)
└── .sicherung/          Sicherung ungespeicherter Änderungen (optional, siehe unten)
```

## Regeln

- **Formatversion:** `format_version` in `projekt.toml` (aktuell 3). Eine andere Version als die
  des Programms wird nicht geladen, sondern gemeldet; ältere Projekte werden nicht migriert (es
  gibt kein Release).
- **Register `szenen.toml` (Formatversion 3):** je Drehinhalt ein `[[szene]]` mit `id` (zufällig,
  8 Zeichen aus `a-z2-7`, nie aus der Nummer gebildet, bleibt für immer), `art`, `number`,
  `heading`, `state`, `prints` und `refused`. `art`: `scene` (Standard), `interview`, `statement`,
  `cutaways` (Schnittbilder), `vignette`, `intro` (Einführung), `vlog`; bei einem Content Item
  ist `heading` der Titel. `state`: `in-book` (Standard, nicht geschrieben) oder `not-in-book`
  (aus dem Buch gestrichen, bleibt mit seinen Floorplans). `prints`: Fingerabdrücke der Elemente
  im Buch, je `"<Art>:<Hash>"` (Zahl, Doppelpunkt, 8 Hexziffern), wie `script/` sie bildet; nicht
  geschrieben, solange leer. Die Folge der Einträge ist die Folge im Projekt. Zu jedem Eintrag
  gehört `szenen/<id>.json` (fehlt sie, hat er keinen Drehort und keinen Plan). Eine Datei ohne
  Eintrag (Absturz beim Speichern, Git-Zusammenführung) lädt: als Eintrag am Ende, ohne Nummer
  und Kopf, `state = "not-in-book"`; das Programm meldet es. `save` schreibt erst die
  Szenen-Dateien, dann `szenen.toml`, dann löscht es Dateien ohne Eintrag. `szenen.toml` lehnt
  unbekannte Schlüssel, unbekannte `art`- und `state`-Werte, ungültige `prints` und `refused`, doppelte
  Ids (auch nur in der Groß-/Kleinschreibung) und Ids, die kein Dateiname sein können, ab;
  `heading` ist eine Zeile. Enthält `szenen.toml` Git-Konfliktmarken (`<<<<<<<` am Zeilenanfang) oder
  `drehbuch.fountain` eine vollständige Konfliktfolge (`<<<<<<<`, `=======`, `>>>>>>>`), lädt
  das Projekt nicht und der Fehler nennt Datei und Zeile. Nummer, Kopf, `state` und `prints`
  folgen dem Buch: Beim Öffnen und Speichern gleicht Klappe die Einträge mit
  `drehbuch.fountain` ab (`script/` `align`, verbunden in `services/`); mit Buch steht die Folge
  im Text, Einträge ohne Szene im Buch (`not-in-book`, offene Fragen, Content Items) folgen
  danach in ihrer Folge. Die `prints` einer Szene des Buchs beginnen mit dem Kopf
  (Art 0); ein Eintrag ohne `prints` kennt noch kein Buch und wird nur zugeordnet, wenn
  Nummer und Kopf beide gleich sind und auf beiden Seiten nur einmal vorkommen, sonst
  gefragt. Eine offene Frage steht nur in der Sitzung. `refused`: Szenen des Buchs, die der
  Eintrag laut Antwort des Nutzers („nicht im Buch“) nicht ist, je 16 Hexziffern über Nummer,
  Kopf und `prints` der Szene; nach ihnen wird nicht wieder gefragt, solange ihr Inhalt
  gleich bleibt. Nicht geschrieben, solange leer; wird geleert, sobald der Eintrag einer
  Szene zugeordnet ist.
- **Papier:** `paper` in `projekt.toml`, z. B. `"a4-portrait"`: Größe (`a4`, `a3`, `letter`),
  Bindestrich, Ausrichtung (`portrait`, `landscape`). Das Blatt, auf dem zuletzt ausgegeben wurde;
  der Ausgabedialog beginnt damit. Fehlt der Schlüssel (und er wird bei `a4-landscape` nicht
  geschrieben), gilt A4 quer; ein unbekannter Teil gilt als sein Standard (`a3` ist A3 quer,
  `tabloid` A4 quer), ein Wert, der kein Text ist, lädt nicht. Keine neue Formatversion: ältere
  Fassungen verlieren ihn beim Speichern.
- **Drehbuchseiten:** Tabelle `[script]` in `projekt.toml` (DB-30, DB-31, DN-05, F-01, F-04);
  fehlt sie, gilt alles Folgende als Standard, und sie wird nicht geschrieben. `language`: `de`
  oder `en`, die Sprache des Projekts; beim Anlegen steht dort die Sprache der Oberfläche, fehlt
  das Feld (ältere Projekte), entscheidet sie.
  `words`: `local` (Standard) oder `international`, die Wörter am Seitenumbruch: `local`
  heißt die der Sprache ((WEITER), (FORTS.) im deutschen, (MORE), (CONT'D) im englischen
  Projekt), `international` immer (MORE), (CONT'D). `paper`: `a4` (Standard) oder `letter`, das
  Papier der Seitenzählung und des Drehbuch-PDFs. `contd_after_action` (`true` oder `false`,
  Standard `false`): (CONT'D) auch, wenn dieselbe Figur nach Handlung weiterspricht. Ein
  unbekannter Wert von `language`, `words` oder `paper` gilt als Standard (wie bei `paper` im
  Plan), ein Wert, der kein Text ist, lädt nicht. Keine neue Formatversion: ältere Fassungen
  verlieren die Tabelle beim Speichern.
  **Szenennummern (DB-33):** `scene_numbering`: `locked` oder `automatic`. Fehlt der
  Schlüssel (Projekte aus der Zeit davor), gilt `locked` und er wird nicht geschrieben; ein
  unbekannter Wert gilt auch als `locked`, damit Klappe nie ungefragt Nummern ändert. Neue
  Projekte schreiben `automatic` ausdrücklich. `first_scene` (Zahl, Standard 1, dann
  nicht geschrieben): die Nummer der ersten Szene bei `automatic`. Das Schlüsselwort ist nicht
  `numbering` der Kopftabelle (das ist das Nummernschema der Shots, LH-76). Die Nummern selbst
  stehen im Fountain-Text (`#12#`) und im Register (`number`); Klappe schreibt sie nur beim
  Speichern in Klappe in den Text, nie beim Öffnen. `automatic`: Szene i hat `first_scene + i`.
  `locked`: Klappe ändert keine Nummer, außer was der Sperrstand
  (`nummern.toml`, unten) für zwei Fälle vorsieht. `automatic` entfernt auch die Köpfe
  `.ENTFÄLLT` / `.OMITTED` (es nummeriert ohnehin neu). Bei `locked` bekommt eine Szene
  ohne Nummer die ihres Vorgängers mit dem nächsten Buchstaben (12, dann 12A, 12B; nach `Z`
  folgt `AA`; Kleinbuchstaben zählen als Großbuchstaben), vor der ersten Nummer `A` und die
  Nummer der nächsten Szene (A1), vor einer Nummer mit Buchstaben `A0`, dann `A0A`; zwischen
  `A1` und `B1` folgt `A1A`. Steht `12A` schon hinter der Stelle, wird es die nächste freie
  Nummer (12B), denn dazwischen sortiert nichts. Eine neue Szene nimmt nie eine Nummer, die im
  Buch schon steht, auch nicht hinter einer Nummer ohne Muster (`I`, `1.1`). Zeilen behalten
  ihr `\r` (CRLF), auch die eingefügten Köpfe `.ENTFÄLLT` mit ihrer Leerzeile.
  **Sperrstand (`nummern.toml`):** die Liste aller nummerierten Köpfe des zuletzt von Klappe
  gespeicherten Buchs in Textfolge, je `[[nummer]]` mit `number` und `id` (die Id des
  Registereintrags, der die Szene hielt; bei einem Kopf `.ENTFÄLLT` nicht geschrieben).
  Nur `Session::save` und `save_script` schreiben sie, zuletzt, nach dem Abgleich des Registers,
  auch bei `automatic`; `Session::create` legt sie leer an. Abgleich, Antworten auf Fragen,
  Sicherung und Wiederherstellen schreiben sie nie, und sie gehört der Sitzung, nicht dem
  Projekt: Rückgängig und die Sicherung tragen sie nicht. Fehlt die Datei, gilt „kein
  Sperrstand“; eine kaputte Datei (Git-Konfliktmarken `<<<<<<<`, ein Wert falschen Typs wie
  `number = 5`) lässt das Projekt nicht laden, wie bei `szenen.toml`; die Meldung nennt Pfad und
  Zeile. Bei
  `locked` gilt: (1) Ohne Sperrstand ändert das Speichern keine Nummer (ein Buch aus der Zeit
  davor, Fountain aus anderen Programmen, bleibt Byte für Byte gleich) und legt nur den
  Sperrstand an. Ebenso, wenn der Sperrstand keine Nummer enthält (ein Buch, das nie
  nummeriert war): auch beim zweiten und jedem weiteren Speichern ändert sich nichts, und eine
  neue Szene darin bekommt keine Nummer. Nummeriert wird ein gesperrtes Buch nur nach einem
  Sperrstand mit Nummern. Ein Kopf `.ENTFÄLLT` / `.OMITTED` ohne Nummer bekommt nie eine. (2) Gestrichen ist eine Nummer des Sperrstands, die im Text weder an einer
  Szene noch an einem Kopf `.ENTFÄLLT` steht: ihr Kopf `.ENTFÄLLT #12#` (`.OMITTED #12#` im
  englischen Projekt, ohne Sprache `ENTFÄLLT`) kommt vor die nächste Nummer des Sperrstands, die
  der Text noch hat, sonst ans Ende; er legt keinen Eintrag im Register an und kennt keinen
  Zustand „war im Buch“, der Eintrag der Szene bleibt mit seinen Plänen „nicht im Buch“.
  (3) Ein Kopf ohne Nummer bekommt eine A-Nummer wie oben. (4) Tragen mehrere Szenen
  dieselbe Nummer N, behält sie die Szene, die der Abgleich dem Eintrag zuordnet, den der
  Sperrstand für N nennt; die anderen gelten als eingefügt und bekommen eine A-Nummer. Gibt es
  keine solche Szene, oder ist eine Szene mit N Kandidat einer offenen Frage, bleibt N überall
  stehen und die Statuszeile meldet es. (5) Nach dem Speichern wird der Sperrstand aus Text und
  Register neu gebildet. Folgen: Speichern ändert nichts, wenn es keinen Sperrstand gibt oder
  der Text ihm entspricht; ein von außen eingefügtes zweites `#2#` wird `2A`, sobald die
  Frage beantwortet ist; eine von Hand von 12 auf 13 geschriebene Nummer ergibt `.ENTFÄLLT
  #12#`; ein von Hand gelöschter Kopf `.ENTFÄLLT` kommt beim nächsten Speichern zurück. Die Sperre je Fassung (DB-40) setzt später
  `locked` zusammen mit den Seiten.
- **Ids:** Drehort- und Szenen-Ids sind Dateinamen und müssen mit der `id` in der Datei
  übereinstimmen. Nicht erlaubt: leer, `/`, `\`, `..`, Beginn mit `.`, Ende auf `.` oder Leerzeichen,
  `<>:"|?*`, Steuerzeichen, Windows-Gerätenamen (`con`, `nul`, `com1` …). Ids, die sich nur
  in der Groß-/Kleinschreibung unterscheiden, gelten als gleich.
- **Reihenfolge:** Drehorte werden nach Id sortiert gelesen. Die Reihenfolge der Szenen steht in
  `szenen.toml`, die der Figuren in `figuren.toml`, die der Shots (Schnittfolge) im Floorplan.
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
- **Kurzname, Storyboard-Bild (LH-110, LH-74):** ein Shot hat optional `short_name` (freier Text
  neben der Nummer, nur geschrieben, wenn nicht leer) und `image` (Pfad relativ zum
  Projektordner, eine Datei in `bilder/`, PNG oder JPEG; nur geschrieben, wenn gesetzt). Die
  Datei wird beim Wählen in `bilder/` kopiert und beim Entfernen des Bildes nicht gelöscht.
  Neue Felder ohne Formatversion: ältere Fassungen verlieren sie beim Speichern.
- **Farbe je Kamera (LH-111):** eine Kamera (`cameras` in `szenen/<id>.json`) hat optional
  `color`, eine der Farben der Figuren (`red`, `orange`, `yellow`, `green`, `cyan`, `blue`,
  `purple`; Kantes Label-Rollen, nie ein Hexwert), nur geschrieben, wenn gesetzt. Fehlt sie,
  gilt die Farbe nach der Stelle der Kamera im Plan (die erste rot, dann orange, gelb, grün,
  cyan, blau, purpur, danach von vorn). Die Nummer eines Shots erscheint in der Farbe seiner
  Kamera; ihr Name steht daneben.
- **Nummernschema und Versionen (LH-76):** `numbering` in `projekt.toml` (nur geschrieben, wenn
  nicht `scene`) wählt, wie neue Shots nummeriert und „Umnummerieren“ rechnet. Ein Shot ist
  eine Version eines anderen, wenn er `version_of` hat (nur geschrieben, wenn gesetzt):
  `{ shot = <id des Basis-Shots im selben Floorplan>, index = n }`, `n` ab 1 (die erste Version
  ist `A`). Ein Basis-Shot hat kein `version_of`; eine Version hat die Kamera ihrer Basis (sie
  entsteht als Kopie; die Kamera einer Version wird nicht einzeln geändert, die der Basis zieht
  ihre Versionen mit). Wird die Basis gelöscht, wird ihre erste Version die neue Basis der
  übrigen. Zeigt `version_of` auf einen Shot, den es nicht gibt oder der selbst
  eine Version ist, zählt der Shot als Basis. `number` ist freier Text und wird nie
  zurückgelesen: er wird beim Anlegen eines Shots oder einer Version und beim Umnummerieren
  aus den Feldern der Shots (Szene, Reihenfolge, Kamera-`label`, Zeitspannen, `version_of`)
  und dem Schema geschrieben; ein Wechsel des Schemas ist deshalb nur Umnummerieren, auch mit
  Versionen. Eine Version heißt Nummer ihrer Basis plus Anhang. Vorhandene Nummern bleiben, bis
  umnummeriert wird; eine neue Nummer zählt die Basis-Shots (der Kamera, der Gruppen) und rückt
  weiter, solange die Nummer vergeben ist.
  - `scene` (Standard): Szene, Punkt, laufende Nummer: `12.3`. Version: Buchstaben dahinter,
    `12.3A`, `12.3B`.
  - `camera`: Szene, Kamerabuchstabe (`label` der Kamera), Punkt, laufende Nummer dieser
    Kamera in der Szene: `12A.3`, `12B.1`. Version: Bindestrich und Buchstaben, `12A.3-B`.
  - `group`: Szene, Punkt, laufende Nummer einer Gruppe gleichzeitig gedrehter Shots, dahinter
    der Kamerabuchstabe: `12.3A`, `12.3B`. Eine Gruppe sind Basis-Shots verschiedener Kameras,
    deren Zeitspannen sich überlappen. Version: Bindestrich und Buchstaben, `12.3A-B`.
  Kein Zeichen hat zwei Bedeutungen: wo Buchstaben Kameras sind, steht vor Versionen ein
  Bindestrich. Eine Szenennummer mit Buchstaben (`12A`) wird unverändert vorangestellt.
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
- **Sonne (LH-52):** `[site]` im Drehort (optional, nur geschrieben, wenn gesetzt): `latitude_deg`
  (Breitengrad, Nord positiv, −90 bis 90), `longitude_deg` (Längengrad, Ost positiv, −180 bis
  180) und `north_deg` (wie weit Norden im Plan im Uhrzeigersinn von oben liegt: 0 Norden oben,
  90 Norden rechts; Standard 0, beim Bearbeiten auf 0 bis unter 360 gebracht). `sun` in
  `szenen/<id>.json` (optional, nur geschrieben, wenn gesetzt): `year`, `month`, `day`, `hour`,
  `minute` als Uhrzeit auf der Uhr der Crew (Jahr 1900 bis 2100) und `utc_offset_min`
  (Minuten, die die Uhr UTC voraus ist, `120` für MESZ; −840 bis 840; fehlt er, gilt
  Mitteleuropäische Zeit mit EU-Sommerzeit: +01:00, und +02:00 vom letzten Sonntag im März
  01:00 UTC bis zum letzten Sonntag im Oktober 01:00 UTC; als feste Regel ohne Zonendatenbank,
  außerhalb Europas wird der Versatz eingetragen). Der Versatz steht in der Datei, weil
  Zonengrenzen offline keine Regel haben und ein Drehplan die Uhrzeit nennt.
  Die Sonne steht im Plan nur, wenn der Drehort `[site]` und die Szene `sun` hat. Ein Ort
  außerhalb der Erdkugel oder ein Datum, das es nicht gibt, lädt nicht (Meldung mit Id). Keine
  neue Formatversion: beides ist optional, ältere Fassungen verlieren es beim Speichern.
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
  schmaler lädt nicht; fehlt sie, gilt 2,4) und optional `rotation_deg` (Grad im Uhrzeigersinn um
  die obere linke Ecke `at`, Standard 0, bei 0 nicht geschrieben; keine endliche Zahl lädt nicht): Karte und Text drehen sich
  zusammen, im Plan und in den Ausgaben. Beschriftungen (`labels`) haben es ebenso. Fehlt der Abschnitt, gibt es keine Notizen; ohne Notizen wird
  er nicht geschrieben, ältere Projekte laden und speichern unverändert. Keine neue
  Formatversion: ein Feld mehr, das ältere Fassungen nicht kennen und beim Speichern verlieren.
- **Beschriftung an ein Element gebunden (LH-118):** eine Notiz mit `bound`, dem Schlüssel
  des Elements, dessen Beschriftung sie ist. Ein Objekt gehört dem Drehort: seine Beschriftung
  ist eine gewöhnliche `[[set.notes]]` mit `bound = "object:<id>"`. Figur und Mark gehören dem
  Plan: ihre Beschriftungen stehen in `labels` im Floorplan (`szenen/<id>.json`), je Eintrag
  wie eine Notiz (`id` `t1`, `t2` …, innerhalb des Plans, `at`, `text`, `width_m`) mit `bound` =
  `figure:<figur>` (die Figur am Abspielkopf) oder `mark:<figur>:<index>` (der Mark mit diesem
  Index, von 0). Sie leben und sterben mit dem Plan: eine Kopie des Plans kopiert sie, das
  Löschen des Plans oder der Szene nimmt sie mit; kein Schlüssel nennt Plan oder Szene. Sie
  werden im Plan, in 3D und in den Ausgaben wie Notizen gezeichnet. `at` bleibt der Ort des
  Kärtchens. Die Bearbeitung verschiebt es um dieselbe Strecke wie das Element; zieht man die
  Notiz selbst, bleibt sie gebunden und das Abstandsmaß ändert sich. Fehlt das Element
  (entfernt) oder ändert sich bei `mark:` die Zahl der Marks der Figur (der Index meint sonst
  einen anderen Mark), wird die Bindung gelöst, die Notiz bleibt stehen. Eine Kopie (Strg+D)
  ist ungebunden. Bei gesperrter Ebene Notizen lehnt die Sitzung eine Bearbeitung ab, die eine
  Notiz oder Beschriftung verschieben oder lösen würde. Ohne Bindung fehlt der Schlüssel, ohne
  Beschriftungen fehlt `labels`; ein unlesbarer Schlüssel bindet nichts. Laden lehnt eine Beschriftung ab, deren Breite unter
  0,6 m liegt oder keine Zahl ist, mit doppelter `id` im Plan, mit einer `id`, die nicht mit `t` beginnt, oder mit
  `bound` ungleich `figure:`/`mark:`. Eine Notiz des Drehorts lädt nicht, wenn ihre `id` nicht mit
  `n` beginnt oder ihr `bound` nicht `object:<id>` ist. So sind die Ids beider Arten nie gleich,
  auch wenn eine Szene an einen anderen Drehort zieht (`Source::Note` nennt beide nur mit der
  `id`). Ältere Projekte laden
  und speichern unverändert, ältere Fassungen verlieren beides beim Speichern. Keine neue
  Formatversion (noch kein Release).
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
- **Skizzen (LH-120):** `sketches` im Floorplan der Szene (`szenen/<id>.json`), je Eintrag ein
  Strich des Skizzenstifts: `points`, eine Liste von Punkten (`x_m`, `y_m` in Metern) in der
  Reihenfolge, in der der Stift sie berührt hat (mindestens zwei; der Stift lässt Punkte unter
  3 cm Abstand aus). Die Striche liegen auf der Ebene „Skizzen“, über allem anderen, haben keine
  Id und sind nicht wählbar; ein Befehl löscht alle. Ohne Skizzen fehlt der Schlüssel; ältere
  Projekte laden und speichern unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Gespeicherte Ansichten (LH-119):** `[[ansicht]]` in `projekt.toml`, je Eintrag `id`
  (eindeutig, ohne Beachtung der Groß-/Kleinschreibung), `name`, `layers` (Namen der sichtbaren
  Ebenen: `set`, `figures`, `cameras`, `lights`, `notes`, `drawings`, `sketches`), `cameras` (Ids der
  sichtbaren Kameras) und `figures` (Ids der Figuren, deren Figur sichtbar ist). Was eine Liste
  nicht nennt, ist beim Wählen der Ansicht aus; Lampen und Ebenen-Sperren und -Abblendung gehören
  nicht dazu. Eine Ansicht im Plan zu wählen ist keine Änderung des Projekts. Fehlen `id` oder
  `name` oder steht eine `id` zweimal, wird das Projekt beim Öffnen mit Meldung abgelehnt. Ohne
  Ansichten fehlt der Abschnitt; ältere Projekte laden und speichern unverändert. Keine neue
  Formatversion (noch kein Release).
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
- **Gerade oder weiche Wege (LH-108):** jeder Mark einer Figur und einer Kamera (`marks` in
  `szenen/<id>.json`) hat optional `curve`, die Form des Wegs vom vorigen Mark hierher:
  `smooth` (Standard, die Kurve aus „Kurvenwege“) oder `straight` (gerade Strecken vom vorigen
  Mark über die Knicke von `via` zu diesem Mark, mit Ecken an den Knicken). Die Bewegung läuft
  auch dann gleichmäßig entlang der Strecken. Ohne Knicke sind beide gleich; `smooth` wird nicht
  geschrieben; am ersten Mark zählt `curve` nicht. Keine neue Formatversion, ältere Dateien
  laden unverändert. Ein Mark, der vor den ersten gesetzt wird (LH-105), ist ein gewöhnlicher
  Mark: der alte erste bekommt den Weg dorthin, seine Zeit und die aller späteren Marks
  verschieben sich um die Laufzeit des neuen Wegs.
- **Auftritt und Abgang (LH-26):** jeder Mark einer Figur (`marks` in `szenen/<id>.json`) hat
  optional `arrival` (`walk` oder `appear`, Standard `walk`) und `departure` (`stay` oder
  `leave`, Standard `stay`). Bei `appear` ist die Figur vor `at_s` dieses Marks nicht zu sehen
  und erscheint dort, statt vom vorigen Mark zu laufen; am ersten Mark heißt das: vor `at_s`
  unsichtbar. Bei `leave` ist sie nach `at_s + hold_s` nicht mehr zu sehen, bis ein späterer Mark
  `appear` hat. Ein Wert mit Standard wird nicht geschrieben; ältere Projekte laden und speichern
  unverändert. Keine neue Formatversion, wie bei den Notizen.
- **Wege von Gegenständen (LH-24):** ein Floorplan (`plans` in `szenen/<id>.json`) hat optional
  `object_tracks`, eine Liste `{ "object": <id>, "marks": [...] }`; `object` ist die Id eines
  Objekts im Set des Drehorts (Verweis nur über die Id, das Objekt bleibt im Set). Ein Mark hat
  `at_s`, `at`, `rotation_deg`, optional `hold_s` (Standard 0) und `carried_by`, die Id einer
  Figur (`character` einer Spur desselben Floorplans): von diesem Mark bis zum nächsten des
  Objekts bewegt diese Figur den Gegenstand, er behält seinen Versatz zu ihr (aus `at` und dem
  Ort der Figur zur Zeit `at_s`, nicht gespeichert), seine Drehung bleibt; ist die Figur dann
  nicht in der Szene, gelten die Marks. Zwischen zwei Marks läuft der Gegenstand gerade, nach der
  Pause, gleichmäßig; vor dem ersten und nach dem letzten steht er auf dem Mark. Am nächsten
  Mark nach einer getragenen Strecke springt er an dessen `at` (er wird dort abgesetzt); ein
  neuer Mark nach einem Trage-Mark entsteht am Abspielkopf dort, wohin der Gegenstand gerade
  getragen wird, ohne Sprung. Ein Objekt ohne
  Spur steht unverändert an seinem Ort im Set; `object_tracks` fehlt, wenn leer. Ein Verweis auf
  ein fehlendes Objekt oder eine fehlende Figur lädt und wirkt nicht (kein Fehler). Keine neue
  Formatversion, ältere Dateien laden unverändert.
- **Spielorte (DB-51):** `orte.toml` hat je Spielort ein `[[ort]]` mit `id` (zufällig, stabil),
  `name` und optional `variants`, andere Schreibweisen des Orts im Szenenkopf (`variants =
  ["SAAL"]`). Der Ort im Kopf ist der Text ohne Präfix (`INT.`, `INNEN` …), Tageszeit (nach dem
  letzten ` - `, ` – ` oder ` — `) und Szenennummer: `INT. SÄNGERSAAL – TAG #12#` hat den Ort
  `SÄNGERSAAL`. Er gehört zum Spielort, der ihn als `name` oder in `variants` trägt (Groß- und
  Kleinschreibung und Leerraum zählen nicht, `ß` gilt wie `SS`). Ein Ort, den kein Spielort trägt,
  wird nicht still angelegt, sondern vorgelegt: neuer Spielort, Variante eines bekannten oder
  Umbenennung eines bekannten (`name` ändert sich, `id` bleibt). Ein Name oder eine Variante
  gehört genau einem Spielort, jede `id` kommt einmal vor, kein Name ist leer; sonst lehnt Laden
  das Projekt ab. Die Datei steht nur, wenn es Spielorte gibt. Eine Antwort schreibt nur diese
  Datei, ausgehend vom Stand auf der Platte (ein Programm daneben, etwa das TUI beim Fenster,
  behält seine Änderungen); Speichern und die Sicherung schreiben sie nie (nur eine Antwort
  tut es; eine Sicherung enthält keine `orte.toml`): der Stand auf der Platte gilt. Eine Frage, die
  eine Antwort schon nicht mehr offen findet, verschwindet nach dem Neulesen der Platte. Antworten
  sind kein Schritt von Rückgängig. Die Zuordnung Spielort → Drehort folgt (Datenmodell Schritt 2). Keine
  neue Formatversion: die Datei fehlt in älteren Projekten und gilt dann als leer.
- **Aliasnamen und Nicht-Figuren (DB-50):** eine Figur in `figuren.toml` hat optional `aliases`,
  eine Liste weiterer Namen, unter denen das Buch sie nennt (`aliases = ["LIESL"]`); nur
  geschrieben, wenn nicht leer, jeder Eintrag nicht leer (Laden lehnt einen leeren ab). Eine
  Figurenzeile des Buchs gehört zur Figur, wenn sie ihren `name` oder einen Alias trägt
  (Groß- und Kleinschreibung zählt nicht, `ß` gilt wie `SS`; Erweiterung `(V.O.)` und Dual-Zeichen `^`
  gehören nicht zum Namen). Ein Name, den keine Figur trägt, wird nicht still zur Figur, sondern
  vorgelegt; die Antwort ist ein Alias oder eine neue Figur oder, für `ALLE` und `STIMME`, ein
  Eintrag in `not_figures`, einer Liste am Anfang von `figuren.toml` vor den `[[figur]]` (nur
  geschrieben, wenn nicht leer). Umbenennen in Klappe ändert `name`, nicht `id` und nicht
  `aliases`. Ein Name oder Alias gehört genau einer Figur: tragen zwei Figuren denselben (Groß- und
  Kleinschreibung zählt nicht), lehnt Laden das Projekt ab. Keine neue Formatversion: Schlüssel,
  die fehlen, gelten als leer. Eine zweite Sitzung (die TUI neben dem Fenster) kann
  `figuren.toml` ändern; darum liest eine Antwort die Datei neu. Eine Antwort auf einen Namen,
  auch ihr Rückgängig und Wiederholen, ändert nur ihren Eintrag (Figur, Alias, `not_figures`).
  Speichern im Fenster übernimmt aus der Datei, was eine Antwort hinzugefügt hat (Figur, Alias,
  `not_figures`), sofern kein Name kollidiert; alles andere überschreibt es. Grenze: macht die
  TUI eine Antwort rückgängig, die ein Speichern im Fenster schon übernommen hat, schreibt das
  nächste Speichern im Fenster sie zurück (die Frage gilt als beantwortet). Wiederherstellen
  einer Sicherung nimmt deren Figuren: eine Antwort, die die TUI vor dem Öffnen des Fensters
  gab, wird wieder gefragt; eine danach gegebene übernimmt das nächste Speichern.
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
- Fehlende `figuren.toml`, `orte.toml`, fehlende Ordner und fehlendes Drehbuch gelten als leer.
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
  Nutzer nur das Drehbuch (TUI: schreibt `drehbuch.fountain` und `szenen.toml`, nicht
  `szenen/<id>.json`; im PDF-Formular nur den Schalter `contd_after_action` in `projekt.toml`), wird die Sicherung entfernt, wenn nur das Drehbuch ungesichert
  war, sonst mit dem gespeicherten Drehbuch neu geschrieben. `load` liest `.sicherung/` nie
  als Teil des Projekts. Git soll die Ordner ignorieren; Vorschlag für `.gitignore` im Projekt:

  ```
  .sicherung/
  .sicherung.neu/
  .sicherung.alt/
  ```

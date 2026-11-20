# Begriffe

Glossar für alle Texte in Klappe (NF-09, [`stellprobe/LASTENHEFT.md`](stellprobe/LASTENHEFT.md)):
deutsche Fassung nach deutscher Filmpraxis, englische Fassung nach englischer. Quelle der
Entscheidungen: [`KONZEPT.md`](KONZEPT.md), die Lastenhefte, [`DATENMODELL.md`](DATENMODELL.md)
(F-01); sie gehen dieser Liste vor. Texte stehen im Katalog `crates/services/i18n/{de,en}.json`.

Stand: 2026-11-18.

## Regeln

- Ein Ding, ein Wort je Sprache. Ein zweites Wort für dasselbe braucht einen Eintrag hier.
- Ein deutsches Wort, das der Film so sagt, steht vor dem Anglizismus; steht der Anglizismus
  im Projekt fest (Shot, Mark, Setup), bleibt er.
- Tasten heißen deutsch wie auf der Tastatur (Strg, Umschalt, Entf), englisch Ctrl, Shift, Del.
- Sichtbare Texte nur über den Katalog, in beiden Sprachen; nie ausgeschrieben in QML oder Rust.

## Glossar

| Deutsch | English | Wo | Anmerkung |
|---|---|---|---|
| Projekt | project | überall | ein Film |
| Szene | scene | überall | |
| Szenenkopf | scene heading (slugline) | TUI, Szenenformular, Drehbuch-PDF | Innen/Außen, Ort, Tageszeit; `INT.`/`INNEN` usw. |
| Drehbuch, kurz „Buch“ | script | Bereich, TUI, Szenenabgleich | „Buch“ nur in den Texten zum Abgleich der Szenen; englisch nie „book“ oder „screenplay“ |
| Fassung | draft | TUI-Titelseite („Fassung vom“ / Draft date) | |
| Spielort | scripted location | Szenenabgleich, `orte.toml` | Ort im Buch (F-01) |
| Drehort | location | Projektliste, Szenenformular | realer Raum mit Set (F-01); auch in Verlaufstexten („Drehort anlegen“), nie nur „Ort“ |
| Set | set | Ebene, Drehort | |
| Floorplan, in der Oberfläche „Plan“ | floor plan, in the UI „plan“ | Bereich Plan, Ausgabe | Lastenheft sagt Floorplan, Oberfläche kurz Plan; Schnappschuss / snapshot für einen festgehaltenen Stand |
| Figur | character (Projekt) / figure (Zeichen im Plan) | überall | deutsch ein Wort; englisch zwei, siehe „Offen“ |
| Besetzung | cast | Figurenkarte | |
| Replik | line | Figurenkarte | Dialogzeile |
| Mark | mark | Plan, Zeitleiste, Karten | Position zu einem Zeitpunkt; „der Mark“; auch in „Voriger / Nächster Mark“ der Zeitleiste |
| Kamera-Mark, Objekt-Mark | camera mark, object mark | Plan, Verlauf | mit Bindestrich |
| Weg | path | Plan | Linie zwischen Marks; englisch nie „way“ |
| Blocking, Inszenieren | blocking | Arbeitsmodus | |
| Auftritt, Abgang | arrival, departure | Markkarte | |
| Pause (am Mark) | hold | Markkarte | englisch „hold“, damit es nicht mit der Pause der Wiedergabe zusammenfällt |
| Shot | shot | überall | Einstellung; im Projekt entschieden |
| Setup | setup | Shotkarte | eine Kameraposition, mehrere Shots |
| Shotliste | shot list | Ausgabe, Bereich | |
| Einstellungsgröße | shot size | Shotkarte | Weit, Totale, Halbtotale, Amerikanisch, Halbnah, Nah, Groß, Detail / extreme wide, wide, medium wide, cowboy, medium, medium close-up, close-up, extreme close-up |
| Perspektive | camera angle | Shotkarte | Normalsicht, Aufsicht, Untersicht, Vogel-, Froschperspektive, Gekippt / eye level, high, low, bird's eye, worm's eye, Dutch angle |
| Bewegung | movement | Shotkarte | Statisch, Schwenk, Neigung, Fahrt, Kran, Handkamera, Gimbal, Zoom / static, pan, tilt, dolly, crane, handheld, gimbal, zoom |
| Achse, Achsensprung | axis, crossing the line | Plan, Shotliste | 180°-Linie |
| Schuss-Gegenschuss | shot and reverse shot | Vorlage | |
| Plansequenz | long take | Vorlage | |
| Kamera | camera | überall | |
| Brennweite | focal length | Kamera, Mark, Shotliste | die Zahl in mm; auch das Shotfeld (nicht „Objektiv“) |
| Objektiv, Objektivsatz | lens, lens set | Kamerakarte | das Glas; Festbrennweiten / primes |
| Sensorbreite, Seitenverhältnis | sensor width, aspect ratio | Kamerakarte | |
| Stativ, Dolly, Schiene, Kran, Handkamera, Gimbal | tripod, dolly, track, crane, handheld, gimbal | Gerät | |
| Gerät | rig | Mark, Shot | Träger der Kamera (Stativ, Dolly …); auch im Verlaufstext („Gerät ändern“) |
| Technik | gear | Shot | |
| Team | crew | Shotliste, Objektgruppe | deutsch „Team“ für Shotfeld und Objektgruppe |
| Requisiten | props | Shot, Objektgruppe | |
| Licht (Ebene, Werkzeug), Lampe (einzelne) | light(s), lamp | Plan | Stufenlinse / Fresnel, LED-Panel, Röhre, Ballon, Reflektor, Abschatter / flag |
| Kamerablick | camera view | Bereich, Ausgabe | |
| 3D-Ansicht | 3D view | Bereich | |
| Ebene | layer | Plan | |
| Zeitleiste | timeline | unten | |
| Schnittfolge, Drehreihenfolge | cut order, shooting order | Shotliste, Zeitleiste | |
| Auflösen | coverage | Arbeitsmodus | Szene in Shots auflösen; „break down“ meint das Drehbuchauszug-Modul |
| Storyboard-Bild | storyboard image | Shot, Verlauf | englisch „image“, nicht „picture“ |
| Bildfolge | image sequence | Ausgabe | |
| Vorlage | template | Plan | |
| Bibliothek | library | Objekte, Lampen | |
| Objekt | object | Plan | auch Möbel, Fahrzeuge; nie „Gegenstand“ |
| Notiz | note | Plan | |
| Maßstab | scale | Werkzeug | |
| Ausgabe, Ausgeben | export | Menü Ausgeben, Fehlertext | PDF, PNG, SVG, Shotliste; „Exportieren“ nur beim Austausch von Bibliothek und Vorlagen als Datei |
| Wiedergabe | playback | Zeitleiste | |
| Sicherung | backup | Start | |
| Rückgängig, Wiederholen | undo, redo | Menü | |
| Drehplan, Dispo, Drehbuchauszug | shooting schedule, call sheet, breakdown | spätere Module | Dispo = Tagesdisposition |
| Szenennummer, A-Szene | scene number, A scene | TUI | |
| Übergang, Handlung, Klammer, Dialog | transition, action, parenthetical, dialogue | TUI | Fountain-Elemente |
| (FORTS.), (WEITER) | (CONT'D), (MORE) | Drehbuch-PDF | je Sprache des Projekts (DN-05, F-04) |
| Strg, Umschalt, Entf | Ctrl, Shift, Del | Hilfen, Tooltips | auch in den deutschen TUI-Texten |

## Offen

Nicht geändert; der Eigentümer entscheidet.

1. **Perspektive „Gekippt“** (Dutch angle): deutsche Praxis sagt eher „Schräge Kamera“ oder
   behält „Dutch Angle“.
2. **„Neigung“** (Bewegung tilt und Mark-Feld): in der Praxis „Neigen“, „Tilt“ oder „Vertikalschwenk“.
3. **„Gerät“** (rig) für Stativ, Dolly, Kran: üblich sind „Kameraträger“ oder „Support“.
4. **„Kamerakörper“** (camera body): üblich „Kamera-Body“ oder „Gehäuse“.
5. **„Figur“ gegen „character“ und „figure“:** deutsch ein Wort für Projektfigur und Zeichen im
   Plan, englisch zwei („Characters“, „Figures“). Englische Praxis kennt nur „character“.
6. **„Plan“ gegen „Floorplan“:** das Lastenheft sagt Floorplan, die Oberfläche Plan.
7. **„Speichern“ gegen „Ablegen“:** Vorlage und Bibliothek heißen „speichern“ (Menü) und „ablegen“
   (Meldung); englisch „save“ und „file away“. Ein Wort wählen.
8. **„Eigene“ / „Own“** für selbst angelegte Objekte und Vorlagen: englisch klingt „Custom“ natürlicher.
9. **Ausgeben gegen Exportieren:** ein Wort für alles, dann „Exportieren“ (wie die CLI).
10. **„Boom operator“** für „Tonangel“ (das Gerät, nicht die Person) und „Director's place“ für
    „Regieplatz“: englisch „boom“, „director's position“.
11. **Kante-Bausteine** (`crates/app/qml/Kante/`) haben deutsche Vorgabetexte (z. B. „Voriger
    Marker“); Klappe überschreibt sie alle. Die Bibliothek bliebe sauberer ohne Vorgabetexte.
12. **`klappe-cli`:** entschieden (Eigentümer, 20.11.2026): Katalog. Die eigenen Fehler kommen
    aus dem Katalog (`cli.*`, Sprache aus `--language`, bei `fov` aus `LANG`). Ausnahmen: die
    Hilfetexte und Meldungen von clap (auch `use one of …` bei `--paper` und `--orientation`,
    geprüft, bevor `--language` gelesen ist) bleiben englisch.
13. **Beispielprojekt** (`crates/demo`, `demo/`): Texte nicht geprüft.

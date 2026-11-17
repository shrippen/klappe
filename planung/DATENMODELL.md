# Datenmodell: Teilbereiche und gemeinsame Daten

Stand: 2026-11-17, **Entwurf**; alle Fragen entschieden (Abschnitt 8).
Zwei Runden zweite Meinung (advisor) eingearbeitet.

Klappe hat mehrere Teilbereiche, die auf einem Projekt arbeiten. Dieses Dokument legt fest,
welche Daten es gibt, welcher Teilbereich sie besitzt (schreibt), wer sie nur liest und wie die
Teilbereiche aufeinander verweisen. Das Wie der Dateien steht danach in `FORMAT.md`, der Aufbau
des Codes in `ARCHITECTURE.md`.

Vorbild für den Umfang der Produktion war Yamdu (Beobachtung am Projekt des Eigentümers,
17.11.2026, nur Verhalten und Begriffe, siehe Anhang A).

## 1. Teilbereiche

| Teilbereich | Zweck | besitzt |
|---|---|---|
| Drehbuch | Buch schreiben, Fassungen, Seiten und Längen | Buchtext, Fassungen; ist es da, die Quelle von Nummer, Kopf und Figurennamen |
| Stellprobe | Floorplans, Blocking, Kameras, Licht, Shots | Sets, Floorplans, Marks, Kameras, Shots |
| Storyboard | Bilder der Shots ansehen, ordnen, ausgeben | nichts Eigenes: Bild und Notiz hängen am Shot, die Reihenfolge ist die Schnittfolge |
| Breakdown | Elemente je Szene für die Departments erfassen und verfolgen | Elemente, Szenenblätter; übernimmt den Drehbuchauszug und die Berichte (DB-60, DB-61) |
| Besetzung/Team | Rollen besetzen, Personen, Departments | Personen, Besetzung, Team, Kontakte (eigene Datei) |
| Drehplan | Szenen auf Drehtage verteilen (Stripboard), Varianten | Drehplanversionen, Drehtage, Streifen |
| Dispo | Tagesdisposition je Drehtag, als Druckvorlage | Dispositionen |

Am Set (Klappe, Take-Protokoll) ist kein eigener Teilbereich; was dort gebraucht wird, sind
Druckvorlagen aus den anderen (Dispo, Shotliste, Storyboard, Floorplan, Sides).
Budget, Finanzierung, Reisen und Postproduktion gehören nicht zu Klappe.

## 2. Grundsätze

1. **Ein Besitzer je Feld.** Jedes Datum schreibt genau ein Teilbereich; die anderen lesen es.
   Wer etwas ändern will, das ihm nicht gehört, öffnet den Besitzer (z. B. die Figur im Buch
   umbenennen, DB-04).
2. **Verweise nur über stabile Ids**, nie über Namen, Nummern, Datum oder Position. Namen und
   Nummern sind Anzeige und ändern sich (Lehre aus #201 und #202: Text-Schlüssel und
   zurückgelesene Nummern gingen beim Umbenennen und Verschieben kaputt).
3. **Neue Ids sind kurz und zufällig**, nicht aus Name oder Zähler gebildet: zwei Personen, die
   in Git gleichzeitig etwas anlegen, erzeugen sonst dieselbe Id für Verschiedenes (auch Marks;
   heutige Zähler-Ids wie `l1`, `n1` werden bei neuen Dingen nicht fortgeführt). Vorhandene Ids
   (`szene-012`, `saengersaal`) bleiben. Listen ohne fachliche Reihenfolge werden nach Id
   sortiert geschrieben; das Register hält seine Folge (siehe Abschnitt 4).
4. **Abgeleitetes wird nicht gespeichert**: Achtel, Seiten, Figuren einer Szene aus dem Buch,
   Drehtage eines Elements, erster Drehtag, Day out of Days. Ausnahme: Das Register merkt sich
   Nummer, Kopf und Fingerabdrücke aus dem Buch beim Speichern, weil der Abgleich nach einer
   fremden Änderung (DB-65) den alten Stand braucht.
5. **Verwaistes lädt und wird gemeldet**, nie still entfernt oder umgebogen (DB-64). `store`
   lehnt Unlesbares und unzulässige Werte ab (wie heute), prüft aber keine Verweise zwischen
   Dateien; diese Prüfung läuft in `services/` (oder als reine
   Funktion in `model/`), weil nur dort Buch und Projekt zusammen vorliegen.
6. **Lieber fragen als falsch zuordnen.** Ein Abgleich, der nicht eindeutig ist, legt dem Nutzer
   vor; eine stille falsche Zuordnung (ein Floorplan unter fremdem Szenenkopf) ist der
   schlimmste Fehler.
7. **Löschen** eines Dings, auf das andere verweisen, fragt nach oder ist gesperrt, solange
   Verweise bestehen (wie heute: eine Figur, die ein Plan nutzt, lässt sich nicht löschen).
8. **Eine Datei je Szene** dort, wo mehrere Departments gleichzeitig schreiben, damit Git selten
   Konflikte meldet.

## 3. Dinge und Besitzer

```
                 Drehbuch (Fountain)
                        │ Szenenköpfe, Figurenzeilen
                        ▼
  Spielort ◄─ Drehinhalt (Register) ──► Figur ◄── Besetzung ──► Person
     │              │      │                                        ▲
     ▼              │      └──► Szenenblatt ──► Element ────────────┘ (verantwortlich)
  Drehort ◄─────────┤          (Breakdown)        ▲
  (Raum mit Set)    ▼                             │ Vorschlag aus Set-Objekten
               Floorplan (hält sein Set) ──► Shot (Bild, Notiz = Storyboard)
                    │
                    ▼
            Drehplan: Drehtag ──► Streifen (Szene) ──► Dispo
```

| Ding | Besitzer | Id | Felder (Auswahl) | gelesen von |
|---|---|---|---|---|
| Drehinhalt (Register: Szene oder Content Item) | Projekt; mit Buch ist das Buch die Quelle von Nummer und Kopf einer Szene (F-07); Klappe führt die Identität (DB-65) | zufällig, stabil | Art (Szene, Interview, …), Nummer, Kopf bzw. Titel, Beschreibung, Zustand (im Buch, nicht im Buch), kurze Fingerabdrücke je Element | alle |
| Szenenblatt | Breakdown | Szenen-Id | Spieltag, Vorstopp, geplante Drehdauer, Continuity, Hinweis für die Produktion, Figuren ohne Dialog (stumm, Komparsen), Element-Zuordnungen, Looks (Kostüm, Maske je Figur), Drehort abweichend | Drehplan, Dispo |
| Figur (Rolle) | Projekt (`figuren.toml`: Name, Kürzel, Farbe, Cast-Nummer, Gruppe, Größe, Aliasnamen); mit Buch kommt der Name aus den Figurenzeilen | stabil | | alle |
| Figur ↔ Szene | abgeleitet: Figurenzeilen + Spuren im Floorplan + Szenenblatt | – | | Drehplan, Day out of Days, Dispo |
| Spielort | Projekt (`orte.toml`); entsteht aus dem Ort im Kopf, mit und ohne Buch (DB-51) | zufällig, stabil | Name, Varianten (andere Schreibweisen im Kopf) | Breakdown, Drehplan |
| Spielort → Drehort | Projekt (Ortsplanung) | – | Vorgabe je Spielort; Abweichung je Szene im Szenenblatt | Stellprobe, Drehplan, Dispo |
| Drehort | Stellprobe (Raum mit Set), Produktion (Adresse, später eigenes Ding) | stabil | Name, Set | Drehplan, Dispo |
| Floorplan | Stellprobe | je Szene stabil | **Set, auf dem er gezeichnet ist** (ersetzt `Scene.location`); Marks (mit Id), Kameras, Licht | Storyboard, Breakdown |
| Shot | Stellprobe | stabil | wie heute; Storyboard-Bild und Notiz | Storyboard, Drehplan, Dispo |
| Element | Breakdown | zufällig | Kategorie, Name, Beschreibung, verantwortliche Person, Status | Dispo, Stellprobe |
| Person | Besetzung/Team | zufällig | Name, Department, Funktion, Agentur | Dispo |
| Kontakt | Besetzung/Team | Personen-Id | Telefon, Mail, Adresse; später weitere persönliche Daten | Dispo |
| Besetzung | Besetzung/Team | – | Figur → Person (vorgeschlagen, fest) | Dispo, Drehplan |
| Drehplanversion | Drehplan | zufällig | Name, aktiv | Dispo |
| Drehtag | Drehplan | zufällig; Datum und Unit sind Felder | Streifen (Szenen in Reihenfolge, Umzug, Pause), Beginn/Ende | Dispo |
| Dispo | Dispo | Drehtag-Id | Zeiten, Ortsangaben, Zeiten je Darsteller, Fahrzeuge, Kontakte, Notizen | – |

**Content Items** (F-07a): Eine Einheit des Drehs, die keine Szene eines Buchs ist: Interview,
Statement, Schnittbilder, Vignette, Einführung, Vlog. Sie steht im selben Register wie die
Szenen, mit der Art als Feld, Titel und Beschreibung statt Kopf, ohne Buchtext und Anker.
Überall, wo eine Szene stehen kann, kann auch ein Content Item stehen: Szenenblatt, Floorplan
und Shots (`szenen/<id>.json`), Streifen im Drehplan, Dispo; alle hängen nur an der Id. Ein
gemeinsamer Typ „Drehinhalt“, damit Drehplan, Breakdown und Dispo nur einen Fall kennen.
Schritt 1 sieht nur das Feld Art vor; die Oberfläche folgt später.

**Drehbuchauszug:** Der Auszug je Szene (DB-60) und die Berichte (DB-61) im Lastenheft Drehbuch
sind der Breakdown. Seine Daten stehen in `breakdown/`, nicht in `drehbuch/tags.toml` (F-02 im
Lastenheft Drehbuch, angepasst: der Fountain-Text bleibt sauber wie entschieden). Besetzung und
Statisterie aus DB-60 sind Figuren der Szene (Figurenzeilen und Szenenblatt), keine Elemente.

**Elementkategorien** (industrieüblich, wie in Yamdu beobachtet): Kostüm, Maskenbild, Bauten,
Ausstattung, Requisite, Grafik, Fahrzeug, Tier, visuelle Effekte, Spezialeffekte, Ton, Musik,
Kamera und Licht, Stunt, Sonstiges. Rollen und Komparsen sind Figuren, keine Elemente. Kostüm und Maske
hängen je Szene an einer Figur („Look“).

**Set-Objekte und Elemente:** Objekte stehen im Set des Drehorts, nicht in der Szene. Ein Objekt
kann auf ein Element zeigen; im Breakdown einer Szene erscheint es dann als **Vorschlag**, den man
übernimmt oder ausschließt, nicht als feste Zuordnung (sonst stünde es in jeder Szene des Orts).

## 4. Verzahnung Drehbuch ↔ Stellprobe (zuerst zu bauen)

- **Szenenidentität (DB-65):** Das Register (`szenen.toml`, Projektebene) hält je Szene Id, Nummer,
  Kopf und kurze Fingerabdrücke je Element (Art und Hash des Textes). Nach einer Änderung des
  Buchs paart Klappe die Fingerabdrücke des alten und des neuen Buchs: eine Zeile, die in beiden
  Büchern genau einmal vorkommt, ist dieselbe, wo auch immer sie steht; gleiche Abschnitte
  zwischen zwei so gepaarten Zeilen werden Stelle für Stelle gepaart. Eine Szene ist
  zugeordnet, wenn mehr als die Hälfte ihrer Zeilen (ohne Figurenzeilen, die in fast jedem
  Dialog stehen) in genau eine neue Szene gepaart sind und umgekehrt. **Nummer und Kopf
  bestätigen nie**: sie nennen nur Kandidaten fürs Vorlegen (eine Nummer ist, was ein Programm
  meinte; Neunummerieren, Kopieren und Einfügen von Szenen mit gleichem Kopf haben jede
  Wertung dafür gegen den Inhalt entscheiden lassen). Alles andere ist mehrdeutig (Kandidaten:
  Szenen mit gepaarten Zeilen, dann ungenommene mit gleichem Kopf oder gleicher Nummer) oder
  nicht gefunden; Zwillinge und Kopien bleiben mehrdeutig, außer das Buch ist um sie herum
  gleich. **Szenen ohne Zeilen** (nur Kopf, oder nur Figurenzeilen) haben nichts außer dem Kopf:
  sie sind zugeordnet, wenn ihr Kopf gepaart ist und die neue Szene ebenfalls keine Zeilen hat;
  wird der Kopf umgeschrieben oder die Szene ausgeschrieben, ist sie mehrdeutig (Kandidaten:
  gleicher Kopf, gleiche Nummer). Floorplans und Shots hängen an der
  Id; Nummer und Kopf stehen nur noch im Register, nicht zusätzlich in `szenen/<id>.json`.
  Derselbe Abgleich zweier Buchstände dient später Revisionen, Vergleich und verschobenen
  Szenen (DB-41, DB-42, DB-44) und steht einmal in `script/` (nur die Suche auf Text;
  `services/` verbindet sie mit dem Register).
- **Reihenfolge:** Mit Buch gilt die Folge im Text; ohne Buch und für Content Items die Folge in
  `szenen.toml` (Ausnahme von der Sortierung nach Id, wie bei `figuren.toml`).
- **Lebenslauf eines Eintrags mit Buch:**
  - „Neue Szene“ in Stellprobe schreibt einen Szenenkopf ins Buch (am Ende oder hinter der
    gewählten Szene); es gibt keinen Szeneneintrag ohne Gegenstück im Buch.
  - Wird eine Szene im Buch gestrichen, bleibt ihr Eintrag mit Floorplans im Zustand „nicht im
    Buch“ und wird gemeldet (Grundsatz 5; DB-33 „ENTFÄLLT“), bis der Nutzer ihn zuordnet oder
    löscht.
  - Bekommt ein Projekt ohne Buch später eines, ordnet Klappe einmal nach Nummer und Kopf zu und
    legt den Rest vor.
- **Konfliktmarken:** Enthält `drehbuch.fountain` oder `szenen.toml` Git-Konfliktmarken
  (`<<<<<<<`), lädt Klappe nicht und meldet es, statt die Marken als Handlung zu lesen und gegen
  Müll abzugleichen. Der Dialog zum Auflösen (F-08) ist ein eigenes Vorhaben.
- **Verweise ins Buch (DB-64):** Heute zeigt ein Mark oder Shot auf „Element Nr. n der Szene“
  (`ScriptLink.element`); das bricht beim Einfügen einer Zeile. Neu: Szenen-Id plus Zitat der
  Zeile (Art, Figur, Text, Vorkommen, Stelle unter den Fingerabdrücken der Szene). In der
  Sitzung folgt der Anker dem Editor; beim Laden nach fremder Änderung paart Klappe die Zeilen
  der alten und der neuen Szene (wie oben): ist die Zeile gepaart und gleich, ist sie es, auch
  verschoben; sonst liegt sie zwischen den Partnern ihrer nächsten gepaarten Nachbarn, und eine
  eindeutig ähnlichste Zeile dort ist es; mehrere sind mehrdeutig, keine ist verschwunden. Gleiche
  Zeilen werden nie allein deshalb zugeordnet, weil sie gleich sind; zwischen unveränderten
  Nachbarn entscheidet ihre Stelle. Liegen gepaarte Zeilen dazwischen (die Zeile ist
  vielleicht umgezogen), eine von mehreren gleichen Zeilen im alten Abschnitt war oder ein
  anderer Sprecher spricht, wird vorgelegt, mit dem Zitat als Hinweis, was gemeint war.
  Mehrere Zitate einer Szene werden zusammen aufgelöst (`find_quotes`): zwei verschiedene
  Zitate auf derselben Zeile werden beide vorgelegt. Beim Speichern wird das Zitat erneuert.
  Keine unsichtbaren Ids im Fountain (Entscheidung F-02 im Lastenheft Drehbuch).
- **Marks bekommen eine Id** (kurz und zufällig); Verweise auf einen Mark nennen sie statt
  der Position in der Spur.
- **Figuren (DB-50):** Die Figurenzeilen des Buchs sind die Quelle der Namen. Ein unbekannter
  Name wird nicht still zur Figur (Tippfehler, „ALLE“, „STIMME“), sondern zum Anlegen oder
  Zuordnen vorgelegt; Zuordnungen werden als Aliasnamen der Figur gemerkt. Umbenennen in Klappe
  (DB-04) behält die Id; fremdes Umbenennen ergibt einen Fall zum Vorlegen.
- **Spielorte und Drehorte (DB-51):** Der Ort im Kopf (aus dem Buch oder in Klappe angelegt) ist ein Spielort.
  Ein Spielort hat eine stabile Id; der Ort im Kopf wird über Name und Varianten zugeordnet.
  Ein unbekannter Ort (auch nach einer Umbenennung im Buch, etwa SÄNGERSAAL → FESTSAAL) wird
  vorgelegt: neuer Spielort, Variante oder Umbenennung eines bekannten. So bleibt die Zuordnung
  zum Drehort bei einer Umbenennung erhalten. Ein Floorplan
  hält das Set fest, auf dem er gezeichnet ist; weicht es von der Ortsplanung ab, meldet Klappe
  das, statt die Marks still in einen anderen Raum zu legen. Drehorte werden nicht automatisch
  angelegt; ein Spielort ohne Zuordnung heißt „noch kein Drehort“.

## 5. Ablage im Projektordner

```
<name>.klappe/
├── projekt.toml              Titel, Sprache, Papier, Ansichten
├── figuren.toml              Figuren (Rollen), Aliasnamen
├── orte.toml                 Spielorte, Varianten, Zuordnung zu Drehorten   (neu)
├── drehorte/<id>.toml        Drehort: Set
├── szenen.toml               Register der Drehinhalte: Szenen, Content Items (neu)
├── drehbuch.fountain         Buchtext (optional, F-07)
├── drehbuch/fassungen/       gesperrte Fassungen (DB-40, F-02 im Lastenheft Drehbuch)
├── szenen/<id>.json          Stellprobe: Floorplans, Shots je Drehinhalt
├── breakdown/elemente.toml   Elemente                                       (neu)
├── breakdown/<szenen-id>.toml Szenenblatt                                   (neu)
├── personen.toml             Personen, Besetzung, Team                     (neu)
├── kontakte.toml             persönliche Daten: Kontakte, Adressen          (neu, Abschnitt 10)
├── drehplan/<id>.toml        eine Drehplanversion: Tage und Streifen       (neu)
├── dispo/<drehtag-id>.toml   Tagesdisposition                              (neu)
└── bilder/                   Hintergründe, Storyboard-Bilder
```

## 6. Schichten im Code

- Die Typen der gemeinsamen Dinge stehen in `model/`; `store/` liest und schreibt alle Dateien
  und lehnt Unlesbares und unzulässige Werte ab, prüft aber keine Verweise zwischen Dateien.
- Die Suche auf Text (Abgleich zweier Buchstände, Zeile zu einem Zitat finden) steht in
  `script/`, der Typ des Zitat-Ankers in `model/`; `services/` verbindet beides.
- Ein neuer Baustein `planung/` neben `stellprobe/` rechnet Drehplan, Day out of Days und Dispo,
  ohne Dateien und Oberfläche. Breakdown und Besetzung sind Listen ohne eigene Rechnung und
  brauchen keinen Baustein.
- `services/` führt die Sitzung über alle Teilbereiche und prüft die Verweise.
- Das ändert „Layers“ in `ARCHITECTURE.md` (neuer Baustein `planung/`) und braucht die
  Bestätigung des Eigentümers in Worten.

## 7. Reihenfolge

1. Register der Drehinhalte (mit Feld Art), Zitat-Anker, Mark-Ids, Figurenabgleich (DB-50,
   DB-64, DB-65); Spielorte mit stabiler Id (`orte.toml`) und das Set am Floorplan; die
   Zuordnung Spielort → Drehort (Rest von DB-51, Muss der ersten Version) folgt in Schritt 2; vorher
   zufällige Ids.
   Zuerst Tests, die heute scheitern (Zeile vor verknüpfter Zeile einfügen, Szene davor
   einfügen, fremd neu nummerieren, Konfliktmarken im Buch).
   `testdata/` und Demo werden im neuen Format geschrieben, `format_version` 3. Vorhandene
   Projekte werden nicht migriert: es gibt kein Release (Eigentümer, 17.11.2026).
2. Breakdown: Elemente, Szenenblatt, Figuren der Szene, Vorschläge aus dem Set; Zuordnung
   Spielort → Drehort (`orte.toml`, Rest von DB-51).
3. Besetzung/Team.
4. Drehplan (Stripboard, Versionen, Day out of Days).
5. Dispo als Druckvorlage.
6. Storyboard-Ansicht über den Shots.

## 8. Entscheidungen und offene Fragen

Entschieden vom Eigentümer am 2026-11-17:

| Nr | Frage | Entscheidung |
|---|---|---|
| F-01 | Begriffe für den Ort im Buch und den realen Ort | „Spielort“ (im Buch) und „Drehort“ (real, Raum mit Set). |
| F-02 | Plant der Drehplan Szenen oder auch Shots/Setups? | Szenen. |
| F-03 | Personen und Kontakte im Projektordner, also in Git | Alle Inhalte eines Projekts sind vertraulich. Wird ein öffentliches Git zur Ablage benutzt, sind die Projektdaten verschlüsselt (Projektpasswort). Persönliche Daten stehen trotzdem in einer eigenen `kontakte.toml`, als Grenze für Rechte (Abschnitt 10). |
| F-04 | Units | Standard eine Unit, mehrere möglich (höchstens 255); Feld am Drehtag. |
| F-05 | Status je Element für die Departments | Ja, einfache Statusliste je Element. |
| F-06 | Projektübergreifende Daten (Darsteller, Fundus) | Nützlich, ausgewählte Informationen global zu machen; vorbereiten für Version 2, nicht jetzt. |
| F-07 | Ist das Drehbuch Pflicht? | Nein: auch Projekte ohne Drehbuch (Reportage, Doku, EB). Das Szenenregister gehört darum dem Projekt: Gibt es ein Buch, ist es die Quelle von Nummer und Kopf; sonst werden Szenen direkt in Klappe angelegt und bearbeitet. |
| F-08 | Automatische Szenennummern vor der Sperre im Text? | Ja, von Anfang an (DB-33 bleibt). Kommt es beim Zusammenführen (Git) zum Konflikt, löst ihn ein Dialog in Klappe auf. |
| F-09 | Neuer Baustein `planung/` in „Layers“ von `ARCHITECTURE.md` | Ja. |
| F-10 | Wie wird verschlüsselt (F-03), ohne Lesbarkeit und Vergleich in Git (NF-03) zu verlieren? | Lokal klar lesbar; verschlüsselt nur beim Ablegen an einem Ziel, dem nicht vertraut wird (Projektpasswort; bei Git z. B. ein Filter wie git-crypt). Je Ziel wählbar: vertraut (privat, Klartext) oder nicht vertraut (öffentlich, verschlüsselt). Siehe Abschnitt 9. Zu klären im Lastenheft-Kapitel: Ein fremdes Git sieht nur verschlüsselte Dateien und kann nicht zeilenweise zusammenführen; Grundsatz 8 und der Dialog aus F-08 brauchen dann einen eigenen Merge-Treiber, der entschlüsselt. Ein Filter wie git-crypt verschlüsselt im Repo selbst; damit bekäme auch ein vertrautes Ziel nur Verschlüsseltes. Vertrauen je Ziel braucht darum je Ziel eine eigene Ablage oder Verschlüsselung beim Übertragen. |
| F-07a | Projekte ohne Szenen (Reportage, Doku, EB) | Content Items gleich als Konzept einplanen (Abschnitt 3). |
| F-11 | Abgleich neben Git | Weitere Wege vorsehen: WebDAV, eventuell p2panda (Abschnitt 9); Vertrauen je Ziel wählbar. |
| F-12 | Hinweise des Advisors (zweite Runde) | Übernehmen; keine Migration vorhandener Projekte, da kein Release. |
| F-13 | Berechtigungen | Nur über einen eigenen, optionalen Klappe-Server; Git und WebDAV ohne Rechte; jetzt nur vorbereiten (Abschnitt 10). |

## 9. Abgleich (Sync)

Ein Projekt lässt sich mit anderen Geräten und Personen abgleichen. Klappe sieht dafür mehrere
Wege vor, hinter einer gemeinsamen Schicht, damit Teilbereiche nichts vom Weg wissen. Sie wird ein
eigener Baustein neben `store/`, festgelegt mit dem Lastenheft-Kapitel Abgleich (ändert dann
„Layers“ in `ARCHITECTURE.md`, mit Bestätigung des Eigentümers):

| Weg | Wofür | Hinweise |
|---|---|---|
| Git | Versionen, Zusammenarbeit mit Verlauf | heute schon möglich (Textformat, NF-03) |
| WebDAV | einfache Ablage auf eigenem Server oder Cloud-Speicher (z. B. Nextcloud) | kein Verlauf und keine Konfliktmarken: Klappe merkt sich den Stand je Datei beim letzten Abgleich und erkennt Konflikte daran |
| Klappe-Server | Zusammenarbeit mit Rechten (Abschnitt 10) | eigenes, selbst betriebenes Programm; gleicht je Änderung ab |
| p2panda | Abgleich von Gerät zu Gerät ohne Server, local-first | zu prüfen: Eignung und Reife (Rust-Bibliothek), Abbildung der Projektdateien |

- **Vertrauen je Ziel** (F-10): Für jedes Ziel wählt der Nutzer „vertraut (privat)“ oder „nicht
  vertraut (öffentlich)“. Nicht vertraute Ziele bekommen nur verschlüsselte Daten (Projektpasswort).
- **Konflikte** löst derselbe Dialog in Klappe, gleich über welchen Weg sie kommen (F-08).
- **Grundsätze 3 und 8** (zufällige Ids, eine Datei je Szene) halten Konflikte bei allen Wegen klein.
- Das Abgleichen selbst ist ein eigenes Vorhaben mit eigenem Lastenheft-Kapitel; Schritt 1 baut
  nur, was es später braucht (Ids, Dateigrenzen, Erkennen von Git-Konfliktmarken); der gemerkte
  Stand je Datei für WebDAV gehört zum Vorhaben Abgleich.

## 10. Berechtigungen (später)

Wer was lesen und schreiben darf (zum Beispiel: alle schreiben im Buch, nur die Produktion sieht
die Kontaktdaten der Darsteller), setzt Klappe nur über den **Klappe-Server** durch
(Eigentümer, 17.11.2026). Git und WebDAV haben keine Rechte: wer das Projekt hat, sieht alles.

- **Server:** liefert jedem nur die Bereiche, die seine Rolle lesen darf, und nimmt nur
  Änderungen an, die sie schreiben darf. Rollen je Teilbereich (z. B. Regie, Produktion,
  Aufnahmeleitung, Department-Leitung, Cast, Gast) mit Rechten lesen/schreiben; der Bereich
  „Persönlich“ getrennt. Konten gibt es nur auf dem selbst betriebenen Server.
- **Jetzt vorbereitet:** Dateigrenzen nach Vertraulichkeit (persönliche Daten in
  `kontakte.toml`, getrennt von `personen.toml`), stabile Personen-Ids, alle Änderungen laufen
  über `services/`.
- **Später:** eigenes Lastenheft für den Server; NF-02 (offline, keine Cloud) und NF-10 (ohne
  Konto) werden dafür neu gefasst und dem Eigentümer vorgelegt: Klappe bleibt offline und ohne
  Konto nutzbar, der Server ist optional und selbst betrieben.

## Anhang A: Yamdu im Verhalten

Beobachtet am eigenen Projekt des Eigentümers, nur lesend; keine Inhalte übernommen.

- Die Szene („Bild“) ist der Knoten: Nummer, Int/Ext, Ort, Tageszeit, Spieltag, Seiten in
  Achteln, Vorstopp, geplante Drehdauer, Drehtag, Unit, abgedreht, gestrichen, Continuity,
  Hinweis für die Produktion; Buchtext je Szene mit Fassung und Revisionsfarbe.
- Elemente je Szene in Kategorien; Kostüm und Maske je Rolle in der Szene („Look“). Elemente
  haben eine Nummer; erster Drehtag und Zahl der Drehtage kommen aus dem Drehplan.
- Ort im Buch und realer Ort sind getrennt; mehrere Orte im Buch werden an einem realen Ort
  gedreht.
- Rollen tragen eine Cast-Nummer, die auf Stripboard und Dispo statt des Namens steht.
- Drehplan in Versionen; Stripboard mit Streifen je Szene, Farbe nach Int/Ext und Tag/Nacht,
  Umzug und Pause als eigene Streifen, Tagessummen (Vorstopp, Seiten, Drehzeit); Kalender daneben.
- Tagesdisposition je Drehtag aus dem Drehplan: Tageszeiten, Ort mit Adresse, Parken,
  Krankenhaus, Szenen mit Besetzung und Elementen, Tabelle je Darsteller (Abholung, Ankunft,
  Maske, Kostüm, Probe, drehfertig), Fahrzeuge, Vorschau auf den nächsten Tag, Kontakte,
  Catering; Status und Versand.
- Day out of Days je Rolle, Ort und Element.

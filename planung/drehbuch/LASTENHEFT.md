# Lastenheft: Drehbuch

Modul der Film-Vorproduktionssuite **Klappe**. Stand: 2026-11-14, Entwurf.

Das Lastenheft beschreibt, *was* das Modul Drehbuch leisten muss. *Wie* es umgesetzt wird,
steht in [`../KONZEPT.md`](../KONZEPT.md) und später im Pflichtenheft. Offene Fragen an den
Eigentümer: Abschnitt 10.

## 1. Ausgangslage

Klappe hat einen Fountain-Parser (`crates/script`) und eine erste TUI (`klappe-tui`, #6, #8). Die
TUI formatiert beim Tippen und zeigt die Szenenliste. Sie speichert nur `drehbuch.fountain`.
Stellprobe verknüpft Drehbuchzeilen mit Marks und Shots (Entwurf B, „Drehbuch daneben“).

Was fehlt, ist ein Schreibwerkzeug auf Augenhöhe mit Fade In und Beat. Es soll unter Linux im
Terminal laufen, Git-tauglich sein und mit Floorplan, Shotliste und später Drehplan verbunden.
Vorbilder, ihre Schwächen und die Clean-Room-Regel: Anhang A.

## 2. Ziel

- Ein Drehbuch im Industrieformat schreiben, überarbeiten und als PDF ausgeben, das wie ein
  Drehbuch aus Final Draft oder Fade In aussieht (Seiten ≈ Minuten).
- Die Fassung bleibt reiner Fountain-Text: lesbar, in Git vergleichbar, von anderen Programmen
  zu öffnen.
- Szenen, Figuren und Drehorte des Drehbuchs sind dieselben wie in Stellprobe und Planung.
  Eine Szene im Drehbuch *ist* die Szene des Floorplans.
- Was die Produktion aus dem Buch braucht (Szenen, Drehorte, Figuren je Szene, Requisiten,
  Länge in Achteln), kommt aus dem Drehbuch, ohne Abtippen.

## 3. Einbettung in Klappe

```
 drehbuch.fountain ──► script (Parser) ──► Szenen, Figuren, Drehorte (model)
        ▲                                        │
   TUI / GUI schreiben                    Stellprobe: Floorplan, Shots
                                          Planung: Drehplan, Dispo (später)
```

- E-09: Die TUI ist der erste Schreibort; die GUI bekommt den Editor später (E-14: Drehbuch
  links neben dem Plan).
- E-08: Das Format ist Fountain. Was Fountain nicht kennt (Revisionen, Tags, Figurendaten),
  steht in eigenen Dateien des Projektordners, nicht im Drehbuchtext (siehe DB-60, Frage F-02).
- Szenennummern verbinden Drehbuch und Plan (`#12#`, `#12A#`, KONZEPT Abschnitt 5).

## 4. Nutzer

| Rolle | Nutzt das Modul für |
|---|---|
| Autor:in / Regie | Schreiben, Umschreiben, Fassungen |
| Regieassistenz | Szenenliste, Auszüge, Länge in Achteln, Breakdown |
| Produktion | Berichte: Drehorte, Figuren, Requisiten je Szene |
| Schauspiel | eigene Rollen-Auszüge (Sides), Vorlesefassung |

Hauptnutzer ist eine Person am Laptop, oft im Terminal.

## 5. Begriffe

| Begriff | Bedeutung |
|---|---|
| Element | Szenenkopf, Handlung, Figur, Klammer, Dialog, Übergang, Zentriert, Liedtext, Seitenumbruch. |
| Szenenkopf | `INT. SÄNGERSAAL – TAG`: Innen/Außen, Ort, Tageszeit. |
| Section | Gliederungsebene über Szenen (Akt, Sequenz); in Fountain `#`. |
| Synopse | Kurzinhalt unter einer Section oder Szene; in Fountain `=`. |
| Fassung | Ein gespeicherter Stand des Drehbuchs mit Namen und Datum. |
| Revision | Änderungen einer Fassung gegenüber der gesperrten Vorfassung, mit Farbe und Randmarke. |
| A-Szene | Eingefügte Szene mit gesperrter Nummer, z. B. 12A zwischen 12 und 13. |
| Achtel | Seitenmaß der Produktion: eine Seite = 8/8. |
| Breakdown | Auszug der Produktionselemente je Szene (Besetzung, Requisite, Kostüm …). |

## 6. Funktionale Anforderungen

Priorität: **M** muss (erste Version), **S** soll (vor 1.0), **K** kann (später).
„TUI“/„GUI“ nennt, wo es zuerst gebraucht wird.

### 6.1 Schreiben

| Nr | Anforderung | Prio |
|---|---|---|
| DB-01 | Fountain tippen mit Formatierung beim Tippen: Elemente erkennen, einrücken, Großschreibung von Szenenköpfen und Figuren (TUI vorhanden). | M |
| DB-02 | Elementwechsel per Tastatur: Enter und Tab wechseln sinnvoll zwischen Handlung, Figur, Klammer, Dialog, Übergang (wie Fade In); erzwungene Elemente per Fountain-Zeichen. | M |
| DB-03 | Autovervollständigung für Figurennamen (häufigste zuerst), Orte, Szenenkopf-Präfixe und Tageszeiten; Figuren-Erweiterungen (V.O.), (O.S.), (CONT'D). | M |
| DB-04 | Figur im ganzen Buch umbenennen (Figurenzeilen, Dual Dialogue, Verweise in Stellprobe). | S |
| DB-05 | Dual Dialogue (`^`), Zentriert, Liedtext, Notizen `[[ ]]`, Boneyard `/* */`, Hervorhebungen. | M |
| DB-06 | Titelseite bearbeiten (Titel, Autor, Kontakt, Fassung, Datum). | M |
| DB-07 | Suchen und Ersetzen, auch nur in Dialog einer Figur. | M |
| DB-08 | Rechtschreibprüfung Deutsch und Englisch mit eigenem Wörterbuch (Figuren- und Ortsnamen gelten als bekannt). | S |
| DB-09 | Rückgängig/Wiederholen ohne feste Grenze; automatisches Sichern wie LH-05. | M |
| DB-10 | Ablenkungsfreier Modus: nur der Text, wahlweise nur der aktuelle Absatz hervorgehoben. | K |

### 6.2 Gliederung und Navigation

| Nr | Anforderung | Prio |
|---|---|---|
| DB-20 | Szenenliste mit Nummer, Kopf, Länge in Achteln; springen per Liste (TUI vorhanden, ohne Länge). | M |
| DB-21 | Gliederung mit Sections und Synopsen, einklappbar; Szenen und Sections per Tastatur verschieben. | M |
| DB-22 | Szenen farbig markieren (Handlungsstrang, Figur, Thema); Farbe nie die einzige Kennzeichnung (Name dazu). | S |
| DB-23 | Karteikarten-Ansicht der Szenen (Kopf, Synopse, Farbe), umsortierbar. | S, GUI |
| DB-24 | Lesezeichen und Notizen im Text, gesammelt in einer Liste. | S |
| DB-25 | Zeitleiste der Handlung (Szenen nach Länge). | K |

### 6.3 Seiten und Länge

| Nr | Anforderung | Prio |
|---|---|---|
| DB-30 | Paginierung nach Industriestandard (US Letter und A4, Courier 12 pt, Ränder nach Standard): Seitenumbrüche wie gedruckt, auch beim Schreiben sichtbar. | M |
| DB-31 | Umbruchregeln: Szenenkopf nie allein am Seitenende; Dialog nur zwischen Sätzen oder an einer Klammer teilen, mit (MORE) unten und Figur (CONT'D) oben; mindestens zwei Zeilen auf jeder Seite; Handlung nur zwischen Sätzen teilen. | M |
| DB-32 | Länge je Szene in Achteln und Gesamtlänge in Seiten und geschätzten Minuten. | M |
| DB-33 | Szenennummern: automatisch oder gesperrt; Startnummer frei; eingefügte Szenen bekommen A-Nummern (12A), gestrichene bleiben als „ENTFÄLLT“ stehen. | M |
| DB-34 | Gesperrte Seiten mit A-Seiten (12A) nach dem Sperren einer Fassung. | S |

### 6.4 Fassungen und Revisionen

| Nr | Anforderung | Prio |
|---|---|---|
| DB-40 | Fassungen benennen und sperren (z. B. „Drehfassung“); eine gesperrte Fassung ist die Basis für Revisionen. | M |
| DB-41 | Revisionen: Änderungen seit der gesperrten Fassung werden markiert. Revisionsfarben in der üblichen Reihenfolge (Blau, Rosa, Gelb, Grün, Goldenrod, …), Randmarke `*` im PDF, Fassungsname und Datum in der Kopfzeile. | S |
| DB-42 | Vergleich zweier Fassungen als Liste und im Text (hinzugefügt, gestrichen, geändert); über Git oder ohne. | S |
| DB-43 | Nur geänderte Seiten drucken (Revisionsseiten). | K |

### 6.5 Figuren, Orte, Auswertung

| Nr | Anforderung | Prio |
|---|---|---|
| DB-50 | Figurenliste aus dem Buch, abgeglichen mit den Figuren des Projekts (Name, Kürzel, Farbe aus Stellprobe, LH-03). | M |
| DB-51 | Drehorte aus Szenenköpfen, abgeglichen mit den Drehorten des Projekts (LH-02); Ortsname im Kopf = Drehort, Varianten zuordenbar. | M |
| DB-52 | Dialog einer Figur gesammelt ansehen (Stimme prüfen). | S |
| DB-53 | Statistik: Szenen und Zeilen je Figur, Dialoganteile, Szenen je Ort und Tageszeit, Szenenlänge. | S |
| DB-54 | Wortwiederholungen anzeigen. | K |

### 6.6 Produktion

| Nr | Anforderung | Prio |
|---|---|---|
| DB-60 | Breakdown-Tags je Szene: Besetzung, Statisterie, Requisite, Kostüm, Maske, Fahrzeug, Tier, Spezialeffekt, Stunt, Ton, Musik, Sonstiges. Der Fountain-Text bleibt sauber (Frage F-02). | S |
| DB-61 | Berichte: Szenen (Nr., Kopf, Länge, Figuren), Drehorte, Figuren je Szene, Breakdown je Szene; als PDF und CSV. Grundlage für das Modul Planung. | S |
| DB-62 | Rollen-Auszüge (Sides): nur die Szenen einer Figur als PDF. | K |
| DB-63 | Szene im Drehbuch ↔ Floorplan, Marks und Shots in Stellprobe springen (beide Richtungen). | M |

### 6.7 Ausgabe und Austausch

| Nr | Anforderung | Prio |
|---|---|---|
| DB-70 | PDF nach Industriestandard (DB-30, DB-31), mit oder ohne Szenennummern, Titelseite, Kopfzeile. | M |
| DB-71 | Fountain lesen und schreiben ohne Verlust (Rundlauf). | M |
| DB-72 | Final Draft (FDX) lesen und schreiben. | S |
| DB-73 | Wasserzeichen je Empfänger im PDF. | K |
| DB-74 | Vorlesen (Text-to-Speech) mit Stimme je Figur. | K |

## 7. Nicht-funktionale Anforderungen

| Nr | Anforderung | Prio |
|---|---|---|
| DN-01 | Läuft im Terminal unter Linux, Windows und macOS (ratatui, E-06); 80 Spalten reichen. | M |
| DN-02 | Ein Buch von 120 Seiten bleibt beim Tippen flüssig: neu paginieren und auswerten ohne spürbare Verzögerung. | M |
| DN-03 | Offline, ohne Konto, ohne Cloud (NF-02). Zusammenarbeit über Git: der Fountain-Text lässt sich in Git zusammenführen. | M |
| DN-04 | Tastatur zuerst; jede Funktion ohne Maus erreichbar, Kürzel in der Hilfe (F1). | M |
| DN-05 | Deutsch und Englisch; Fachbegriffe nach Filmpraxis (NF-09); Szenenkopf-Präfixe beider Sprachen erkennen (INT./INNEN, EXT./AUSSEN, TAG/NACHT). | M |
| DN-06 | Kein Datenverlust bei Absturz (LH-05, Sicherung wie Stellprobe). | M |
| DN-07 | Das PDF stimmt Seite für Seite mit der Paginierung im Editor überein. | M |

## 8. Abgrenzung

- Kein eigenes Binärformat, kein Cloud-Dienst, keine Echtzeit-Zusammenarbeit (Git statt dessen).
- Keine Drehplanung und Dispo (Modul Planung); das Drehbuch liefert nur die Daten.
- Kein Romanformat und kein Bühnenformat in der ersten Version.
- Kein Import geschlossener Formate außer FDX (offen dokumentiertes XML).

## 9. Abnahme

Die erste Version ist abgenommen, wenn

1. das Drehbuch des Wartburgfilms in der TUI geschrieben und überarbeitet werden kann,
2. das PDF Seite für Seite mit einer Referenz aus einem Standardprogramm übereinstimmt
   (± eine Zeile je Seite) und (MORE)/(CONT'D) richtig setzt,
3. jede Szene des Buchs mit ihrem Floorplan in Stellprobe verbunden ist (DB-63),
4. alle Muss-Anforderungen erfüllt sind.

## 10. Offene Fragen an den Eigentümer

| Nr | Frage | Vorschlag |
|---|---|---|
| F-01 | Papierformat: US Letter (Industriestandard, ~1 Seite/Minute) oder A4 als Standard? | A4 für deutsche Produktionen, Letter wählbar; Minutenschätzung je Format. |
| F-02 | Wo stehen Revisionen, Tags, gesperrte Szenen-/Seitennummern? Beat schreibt sie als JSON-Block ans Ende der Fountain-Datei (in einem Kommentar). | Eigene Dateien im Projektordner (`drehbuch/fassungen/`, `drehbuch/tags.toml`), Fountain bleibt sauber; Szenen per Szenennummer referenziert. |
| F-03 | Revisionen: eigene Markierung wie Fade In/Beat, oder aus Git abgeleitet (Diff gegen die gesperrte Fassung)? | Gegen eine gespeicherte gesperrte Fassung im Projektordner diffen; Git optional. |
| F-04 | Englische oder deutsche Elemente im PDF ((MORE)/(CONT'D) vs. (WEITER)/(FORTS.))? | Je Projekt wählbar, Standard nach Sprache des Projekts. |
| F-05 | Reihenfolge nach der ersten Version: GUI-Editor (DB-23 Karteikarten) oder Produktion (DB-60, DB-61)? | Erst Produktion, weil Planung darauf aufbaut. |

## Anhang A: Analyse Fade In und Beat

### Clean-Room-Regel

Übernommen wird, *was* die Vorbilder leisten. Code, Texte, Grafiken, Vorlagen und eigene
Dateiformate werden nicht übernommen. Klappe steht unter MIT.

- **Fade In** (proprietär, GCC Productions): nur öffentliche Seiten.
- **Beat** (GPL v3, Lauri-Matti Parppei): öffentliche Seiten. Auf Wunsch des Eigentümers
  (2026-11-14) wurden auch einzelne Quelldateien gelesen, um Verhalten zu verstehen
  (Paginierung, Revisionen, Dokumenteinstellungen, Tagging, Autovervollständigung). Daraus
  steht hier nur Verhalten in eigenen Worten. Wer Klappe programmiert, schreibt nach diesem
  Lastenheft, nicht mit Beat-Code daneben.

Gelesen (2026-11-14):

- fadeinpro.com: Startseite, `page.pl?content=features`, Hilfe-Artikel „Track Changes“;
  Mac-App-Store-Text; Rezensionen nur als Suchauszug (G2, Capterra, StudioBinder, Reddit).
- Beat: github.com/lmparppei/beat (README, Wiki), beat-app.fi, App-Store-Text; Quelldateien
  `Frameworks/BeatPagination2/…/Pagination/` (BeatPagination, BeatPaginationBlock,
  BeatPaginationBlockGroup, BeatPageBreak), `Frameworks/BeatCore/…/Revisions/BeatRevisions.m`,
  `Frameworks/BeatParsing/…/BeatDocumentSettings.m`, `Frameworks/BeatCore/…/Tagging/BeatTagging.m`,
  `Frameworks/BeatCore/…/Editor/BeatAutocomplete.m`, Statistik-Plugin.

### Funktionsumfang

| Bereich | Fade In | Beat | Anforderung |
|---|---|---|---|
| Plattformen | Mac, Windows, Linux; mobil über „Fade In Access“ | macOS, iOS | DN-01 |
| Formatierung beim Tippen | Wechsel Szenenkopf/Handlung/Dialog automatisch, Stile anpassbar | ja, Fountain-Zeichen ausblendbar | DB-01, DB-02 |
| Autovervollständigung | Figuren, Orte, Szenenkopf-Präfixe, Tageszeiten; Listen pflegbar; Figur im ganzen Buch umbenennen | Figuren (nach Häufigkeit, Erweiterungen wie (V.O.) abgetrennt, Erweiterungen als Vorschlag, wenn kein Name passt), Szenenköpfe (Präfix bekommt den Punkt) | DB-03, DB-04 |
| Gliederung | Navigator: springen, umsortieren, verschachtelte Sequenzen, Farben, Synopsen, Lesezeichen | Gliederung mit Sections/Synopsen, Index Cards, Timeline, Farben | DB-20–DB-25 |
| Ablenkungsfrei | Vollbild, nur aktueller Abschnitt | schlichte Oberfläche | DB-10 |
| Paginierung | Industriestandard, (MORE)/(cont'd) änderbar, Zeilenlängen-Feinabgleich | siehe unten | DB-30, DB-31 |
| Revisionen | Fassungen, Szenen- und Seitensperre, gestrichene Szenen, Revisionsfarben bis in die Vorproduktion, Versionsvergleich | Revisionsgenerationen mit Farbe und Marke, gesperrte Szenennummern | DB-33, DB-34, DB-40–DB-43 |
| Dialog prüfen | Dialogue Tuner (alle Zeilen einer Figur), Häufigkeit von Wörtern | Statistik (Zeilen je Figur, Orte, Tageszeiten, Szenenlängen, Geschlechterverteilung) | DB-52–DB-54 |
| Breakdown | Standard-Breakdowns, Berichte (Szenen, Besetzung, Drehorte) als Druck, HTML, CSV; anpassbare Tags | Tags mit Kategorien (siehe unten) | DB-60, DB-61 |
| Austausch | eigenes XML-Format (Open Screenplay Format), FDX, FDR (lesen), DOCX, RTF, Fountain, Celtx, Scrivener, Text, HTML, PDF, Wasserzeichen-Stapel | Fountain, FDX, PDF | DB-70–DB-73 |
| Weiteres | Bilder im Dokument, Haftnotizen, Rechtschreibung mit eigenen Wörterbüchern, Formatprüfung, Vorlesen (Table Read) | Plugins in JavaScript, Notizen, Sicherungen | DB-08, DB-24, DB-74 |

### Beat im Verhalten (aus dem Code, in eigenen Worten)

- **Paginierung:** Das Buch wird in Blöcke geteilt (ein Dialogblock = Figur mit Klammern und
  Dialog; ein Szenenkopf zieht den folgenden Block mit sich). Passt ein Block nicht mehr auf
  die Seite, wird er nur an sicheren Stellen geteilt: Dialog an einer Klammer oder zwischen
  Sätzen (Satzgrenzen per Satzzeichen, notfalls Wörter, bei einem überlangen Wort Zeichen).
  Am Seitenende steht dann (MORE), oben die Figur mit (CONT'D). Bleiben weniger als zwei Zeilen
  auf der Seite, wandert der Block ganz nach vorn. Handlung wird ebenfalls zwischen Sätzen
  geteilt. Erzwungene Seitenumbrüche gelten zuerst. Nach einer Änderung wird ab der ersten
  betroffenen Seite neu paginiert, die Seiten davor bleiben.
- **Revisionen:** Geänderte Textbereiche tragen eine Generation (0, 1, 2 …), jeder Generation
  ist eine Farbe und eine Randmarke zugeordnet (Blau `*`, Rosa `**`, Gelb `+`, Grün `++`, …).
  Gespeichert werden Bereiche (Anfang, Länge, Generation) getrennt nach hinzugefügt,
  gestrichen, Streichvorschlag.
- **Metadaten:** Beat hängt Einstellungen, Revisionsbereiche, Tags, Figurendaten, gesperrte
  Nummern und Fensterzustand als JSON in einem Kommentarblock ans Ende der Fountain-Datei.
  Andere Programme sehen das als Boneyard; ein Diff in Git zeigt bei jeder Änderung den Block.
  Darum Frage F-02.
- **Tagging:** Feste Kategorien (Besetzung, Requisite, Spezialeffekt, Toneffekt, Kamera, Tier,
  Statisterie, Fahrzeug, Kostüm, Maske, Musik, Ton, Stunt, Sonstiges), je mit Farbe. Ein Tag
  verweist auf eine Definition (Name, Kategorie) und auf einen Textbereich; daraus entsteht
  ein Bericht je Kategorie.
- **Statistik:** Zeilen und Wörter je Figur, Verteilung nach Geschlecht (vom Nutzer gesetzt),
  Szenen je Ort und Tageszeit, als Plugin mit eigener Ansicht.

### Schwächen laut Nutzern

| Produkt | Schwäche | Quelle |
|---|---|---|
| Fade In | gelegentliche Abstürze, Sicherungen schwer zu finden | G2 (Suchauszug) |
| Fade In | keine Echtzeit-Zusammenarbeit; Mobil getrennt zu kaufen | StudioBinder-Vergleich |
| Fade In | versehentlich gesperrte Seiten ergeben unerwartete Nummern | Reddit r/Screenwriting, 01/2025 |
| Beat | nur Apple-Plattformen | beat-app.fi |
| Beat | kein Abgleich zwischen Geräten, Notizen ohne Anhänge | App-Store-Rezensionen |
| Beat | Metadaten im Fountain-Text stören Diffs (eigene Beobachtung aus dem Verhalten) | – |

### Verbesserungen in Klappe

| Nr | Verbesserung | Anforderung |
|---|---|---|
| 1 | Läuft unter Linux, Windows, macOS, im Terminal | DN-01 |
| 2 | Drehbuch und Floorplan sind dieselben Szenen; Sprung in beide Richtungen | DB-63 |
| 3 | Fountain bleibt sauber, Metadaten in eigenen Dateien; Git-Diffs zeigen nur Text | F-02, DN-03 |
| 4 | Breakdown und Berichte fließen ohne Export in Planung | DB-60, DB-61 |
| 5 | Kein Datenverlust bei Absturz, Sicherung wie Stellprobe | DN-06 |

# Lastenheft: Drehbuch

Modul der Film-Vorproduktionssuite **Klappe**. Stand: 2026-11-14, Entwurf.

Das Lastenheft beschreibt, *was* das Modul Drehbuch leisten muss. *Wie* es umgesetzt wird,
steht in [`../KONZEPT.md`](../KONZEPT.md) und später im Pflichtenheft. Entscheidungen des
Eigentümers: Abschnitt 10.

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
- Szenen verbinden Drehbuch und Plan; Klappe führt die Zuordnung (DB-65), die Nummer (`#12#`, `#12A#`,
  KONZEPT Abschnitt 5) ist Anzeige und Bezeichner.

## 4. Nutzer

| Rolle | Nutzt das Modul für |
|---|---|
| Autor:in / Regie | Schreiben, Umschreiben, Fassungen |
| Regieassistenz | Szenenliste, Auszüge, Länge in Achteln, Drehbuchauszug |
| Produktion | Berichte: Drehorte, Figuren, Requisiten je Szene |
| Schauspiel | eigene Rollen-Auszüge (Sides), Vorlesefassung |

Hauptnutzer ist eine Person am Laptop, oft im Terminal.

## 5. Begriffe

| Begriff | Bedeutung |
|---|---|
| Element | Szenenkopf, Handlung, Figur, Klammer, Dialog, Übergang, Zentriert, Liedtext, Seitenumbruch. |
| Szenenkopf | `INT. SÄNGERSAAL – TAG`: Innen/Außen, Ort, Tageszeit. |
| Abschnitt | Gliederungsebene über Szenen (Akt, Sequenz); in Fountain `#` (engl. Section). |
| Synopse | Kurzinhalt unter einem Abschnitt oder einer Szene; in Fountain `=`. |
| Fassung | Ein gespeicherter Stand des Drehbuchs mit Namen und Datum. |
| Revision | Änderungen einer Fassung gegenüber der gesperrten Vorfassung, mit Farbe und Randmarke. |
| A-Szene | Eingefügte Szene mit gesperrter Nummer, z. B. 12A zwischen 12 und 13. |
| Achtel | Seitenmaß der Produktion: eine Seite = 8/8. |
| Drehbuchauszug | (engl. Breakdown) Auszug der Produktionselemente je Szene (Besetzung, Requisite, Kostüm …). |

## 6. Funktionale Anforderungen

Priorität: **M** muss (erste Version), **S** soll (vor 1.0), **K** kann (später).
„TUI“/„GUI“ nennt, wo es zuerst gebraucht wird.

### 6.1 Schreiben

| Nr | Anforderung | Prio |
|---|---|---|
| DB-01 | Fountain tippen mit Formatierung beim Tippen: Elemente erkennen, einrücken, Großschreibung von Szenenköpfen und Figuren (TUI: Erkennen und Einrücken vorhanden, Großschreibung fehlt). | M |
| DB-02 | Elementwechsel per Tastatur: Enter und Tab wechseln zwischen Handlung, Figur, Klammer, Dialog, Übergang (wie Fade In; Tabelle im Pflichtenheft); erzwungene Elemente per Fountain-Zeichen. Tab schaltet in der TUI heute den Fokus um und wird dafür frei. | M |
| DB-03 | Autovervollständigung für Figurennamen (häufigste zuerst), Orte, Szenenkopf-Präfixe und Tageszeiten; Figuren-Erweiterungen deutsch und englisch, z. B. (OFF), (V.O.), (O.S.), (CONT'D). | M |
| DB-04 | Figur im ganzen Buch umbenennen (Figurenzeilen, Doppeldialog, Verweise in Stellprobe). | S |
| DB-05 | Doppeldialog (`^`), Zentriert, Liedtext, Notizen `[[ ]]`, auskommentierter Text `/* */`, Hervorhebungen. | M |
| DB-06 | Titelseite bearbeiten (Titel, Autor, Kontakt, Fassung, Datum). | M |
| DB-07 | Suchen und Ersetzen, auch nur in Dialog einer Figur. | M |
| DB-08 | Rechtschreibprüfung Deutsch und Englisch mit eigenem Wörterbuch (Figuren- und Ortsnamen gelten als bekannt). | S |
| DB-09 | Rückgängig/Wiederholen ohne feste Grenze. | M |
| DB-10 | Ablenkungsfreier Modus: nur der Text, wahlweise nur der aktuelle Absatz hervorgehoben. | K |

### 6.2 Gliederung und Navigation

| Nr | Anforderung | Prio |
|---|---|---|
| DB-20 | Szenenliste mit Nummer, Kopf, Länge in Achteln; springen per Liste (TUI vorhanden, ohne Länge). | M |
| DB-21 | Gliederung mit Abschnitten (Akt, Sequenz) und Synopsen, einklappbar; Szenen und Abschnitte per Tastatur verschieben. | M |
| DB-22 | Szenen farbig markieren (Handlungsstrang, Figur, Thema); Farbe nie die einzige Kennzeichnung (Name dazu). | S |
| DB-23 | Karteikarten-Ansicht der Szenen (Kopf, Synopse, Farbe), umsortierbar. | S, GUI |
| DB-24 | Lesezeichen und Notizen im Text, gesammelt in einer Liste. | S |
| DB-25 | Zeitleiste der Handlung (Szenen nach Länge). | K |

### 6.3 Seiten und Länge

| Nr | Anforderung | Prio |
|---|---|---|
| DB-30 | Paginierung nach Industriestandard (A4 als Standard, US Letter wählbar (F-01); Courier 12 pt, Ränder nach Standard): Seitenumbrüche im Editor sichtbar (DN-07). | M |
| DB-31 | Umbruchregeln: Szenenkopf nie allein am Seitenende; Dialog nur zwischen Sätzen oder an einer Klammer teilen, mit (MORE) unten und Figur (CONT'D) oben; mindestens zwei Zeilen auf jeder Seite; Handlung nur zwischen Sätzen teilen. | M |
| DB-32 | Länge je Szene in Achteln und Gesamtlänge in Seiten und geschätzten Minuten. | M |
| DB-33 | Szenennummern: automatisch oder gesperrt; Startnummer frei; eingefügte Szenen bekommen A-Nummern (12A), gestrichene bleiben als „ENTFÄLLT“ stehen. Die Nummer steht im Fountain-Text (`#12#`). Ändert sich das Buch (in Klappe oder extern, z. B. in einem anderen Programm oder per Git), passt Klappe die automatischen Nummern nach diesen Regeln an, schreibt sie beim Speichern in den Fountain-Text und führt Floorplan, Marks und Shots jeder Szene mit (DB-65); gesperrte Nummern bleiben, bis die Sperre aufgehoben wird. | M |
| DB-34 | Gesperrte Seiten mit A-Seiten (12A) nach dem Sperren einer Fassung. | S |

### 6.4 Fassungen und Revisionen

| Nr | Anforderung | Prio |
|---|---|---|
| DB-40 | Fassungen benennen und sperren (z. B. „Drehfassung“); eine gesperrte Fassung ist die Basis für Revisionen. | M |
| DB-41 | Revisionen: Änderungen seit der gesperrten Fassung werden markiert (Vergleich mit der beim Sperren gespeicherten Fassung, F-03). Revisionsfarben in der üblichen Reihenfolge (Blau, Rosa, Gelb, Grün, Goldgelb, …), Randmarke `*` im PDF, Fassungsname und Datum in der Kopfzeile. | S |
| DB-42 | Vergleich zweier Fassungen als Liste und im Text (hinzugefügt, gestrichen, geändert); über Git oder ohne. | S |
| DB-43 | Nur geänderte Seiten drucken (Revisionsseiten). | K |
| DB-44 | Verschobene Szenen erkennen: eine Szene, die nur ihren Platz gewechselt hat, gilt als verschoben, nicht als gestrichen und neu eingefügt; ihr Text bleibt unmarkiert. | S |

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
| DB-60 | Drehbuchauszug (Tags) je Szene: Besetzung, Statisterie, Requisite, Kostüm, Maske, Fahrzeug, Tier, Spezialeffekt, Stunt, Ton, Musik, Sonstiges. Der Fountain-Text bleibt sauber (Frage F-02). | S |
| DB-61 | Berichte: Szenen (Nr., Kopf, Länge, Figuren), Drehorte, Figuren je Szene, Drehbuchauszug je Szene; als PDF und CSV. Grundlage für das Modul Planung. | S |
| DB-62 | Rollen-Auszüge (Sides): nur die Szenen einer Figur als PDF. | K |
| DB-63 | In der GUI von der Szene im Drehbuch zu Floorplan, Marks und Shots in Stellprobe springen und zurück. Die TUI ist vorerst nur zum Schreiben des Drehbuchs. | M, GUI |
| DB-64 | Verknüpfungen aus Stellprobe (Szene, Szenenkopf, verknüpfte Drehbuchzeilen von Marks und Shots) bleiben beim Bearbeiten, Verschieben und Streichen gültig; was sich nicht eindeutig zuordnen lässt, legt das Programm dem Nutzer zur Zuordnung vor, statt still auf eine andere Zeile zu zeigen. | M |
| DB-65 | Szenen behalten ihre Identität über Änderungen am Buch: Klappe merkt sich beim Speichern jede Szene (Nummer, Kopf, Inhalt) im Projektordner und ordnet beim nächsten Öffnen die Szenen des Buchs diesen zu, auch nach einer Änderung außerhalb von Klappe. Eine umnummerierte, verschobene oder umgeschriebene Szene behält ihren Floorplan; was sich nicht eindeutig zuordnen lässt, legt das Programm dem Nutzer vor (wie DB-64). Der Fountain-Text bleibt dabei sauber (F-02). | M |

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
| DN-02 | Tippen wird nie durch Rechnen gebremst: Seitenumbrüche, Längen und Auswertung rechnet Klappe neu, wenn 5 s lang keine Taste gedrückt wurde, und vor Export, Druck und Speichern (DN-07); dazwischen dürfen sie veraltet sein. | M |
| DN-03 | Offline, ohne Konto, ohne Cloud (NF-02). Zusammenarbeit über Git: der Fountain-Text lässt sich in Git zusammenführen. | M |
| DN-04 | Tastatur zuerst; jede Funktion ohne Maus erreichbar, Kürzel in der Hilfe (F1). | M |
| DN-05 | Deutsch und Englisch; Fachbegriffe nach Filmpraxis im Buch und im PDF nach der Sprache des Projekts (wie F-04): ein englisches Projekt nutzt die internationalen Begriffe, ein deutsches die deutschen oder auf Wunsch die internationalen (je Projekt wählbar); die Oberfläche folgt NF-09; Szenenkopf-Präfixe beider Sprachen erkennen (INT./INNEN, EXT./AUSSEN, TAG/NACHT). | M |
| DN-06 | Kein Datenverlust bei Absturz: automatisches Sichern wie LH-05. | M |
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
   (Seitenumbrüche an denselben Stellen; Referenz in A4 nach F-01) und
   (MORE)/(CONT'D) richtig setzt,
3. sich jede Szene mit einem Floorplan verbinden lässt und die Verbindung Umschreiben
   übersteht (DB-63 bis DB-65),
4. alle Muss-Anforderungen erfüllt sind.

## 10. Fragen an den Eigentümer

Entschieden vom Eigentümer am 2026-11-14:

| Nr | Frage | Entscheidung |
|---|---|---|
| F-01 | Papierformat | A4 ist Standard, US Letter wählbar; Minutenschätzung je Format. |
| F-02 | Wo stehen Revisionen, Tags, gesperrte Seitennummern? (Beat: JSON-Block am Ende der Fountain-Datei) | Eigene Dateien im Projektordner (`drehbuch/fassungen/`, `drehbuch/tags.toml`); der Fountain-Text bleibt sauber. Szenennummern bleiben im Text (DB-33); die Zuordnung zu Floorplan, Marks und Shots führt Klappe getrennt von der Nummer (DB-65). |
| F-03 | Woher kommen Revisionen: Markierung beim Tippen (Fade In, Beat), Git oder Vergleich? | Vergleich mit einer beim Sperren gespeicherten Fassung im Projektordner; Git optional. Verschobene Szenen werden eigens erkannt (DB-44). |
| F-04 | (MORE)/(CONT'D) oder (WEITER)/(FORTS.) im PDF | Je Projekt wählbar, Standard nach Sprache des Projekts. |
| F-05 | Nach der ersten Version: GUI-Editor (DB-23) oder Produktion (DB-60, DB-61)? | Erst Produktion, weil Planung darauf aufbaut. |
| DB-33 | Ändern sich Szenennummern mit Floorplan nie automatisch? | Nein: sie passen sich an, wenn sich das Buch ändert, auch extern; Klappe verfolgt die Szenen (DB-65). |
| DB-64 | Verknüpfungen überstehen das Umschreiben | Einverstanden. |
| Begriffe | Deutsch nach Filmpraxis | Einverstanden, mit Mehrsprachigkeit: ein englisches Buch hat internationale Begriffe, ein deutsches kann sie auf Wunsch auch haben; maßgeblich ist die Sprache des Projekts (DN-05). |
| DB-63 | Springen nur in der GUI? | Ja; die TUI ist vorerst nur zum Drehbuchschreiben. |
| DN-02 | Neu berechnen nach jedem Tastendruck (50 ms)? | Nein: erst wenn 5 s keine Taste gedrückt wurde (und vor Export, Druck, Speichern). |

## Anhang A: Analyse Fade In und Beat

### Clean-Room-Regel

Übernommen wird, *was* die Vorbilder leisten. Code, Texte, Grafiken, Vorlagen und eigene
Dateiformate werden nicht übernommen. Klappe steht unter MIT.

- **Fade In** (proprietär, GCC Productions): nur öffentliche Seiten.
- **Beat** (GPL v3, Lauri-Matti Parppei): öffentliche Seiten. Auf Wunsch des Eigentümers
  (2026-11-14, bestätigt in einem Kommentar zu #17) wurden auch einzelne Quelldateien gelesen, um Verhalten zu verstehen
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
| Gliederung | Navigator: springen, umsortieren, verschachtelte Sequenzen, Farben, Synopsen, Lesezeichen | Gliederung mit Abschnitten/Synopsen, Index Cards, Timeline, Farben | DB-20–DB-25 |
| Ablenkungsfrei | Vollbild, nur aktueller Abschnitt | schlichte Oberfläche | DB-10 |
| Paginierung | Industriestandard, (MORE)/(cont'd) änderbar, Zeilenlängen-Feinabgleich | siehe unten | DB-30, DB-31 |
| Revisionen | Fassungen, Szenen- und Seitensperre, gestrichene Szenen, Revisionsfarben bis in die Vorproduktion, Versionsvergleich | Revisionsgenerationen mit Farbe und Marke, gesperrte Szenennummern | DB-33, DB-34, DB-40–DB-43 |
| Dialog prüfen | Dialogue Tuner (alle Zeilen einer Figur), Häufigkeit von Wörtern | Statistik (Zeilen je Figur, Orte, Tageszeiten, Szenenlängen, Geschlechterverteilung) | DB-52–DB-54 |
| Drehbuchauszug | Standard-Breakdowns, Berichte (Szenen, Besetzung, Drehorte) als Druck, HTML, CSV; anpassbare Tags | Tags mit Kategorien (siehe unten) | DB-60, DB-61 |
| Austausch | eigenes XML-Format (Open Screenplay Format), FDX, FDR (lesen), DOCX, RTF, Fountain, Celtx, Scrivener, Text, HTML, PDF, Wasserzeichen-Stapel | Fountain, FDX, PDF | DB-70–DB-73 |
| Weiteres | Bilder im Dokument, Haftnotizen, Rechtschreibung mit eigenen Wörterbüchern, Formatprüfung, Vorlesen (Table Read) | Plugins in JavaScript, Notizen, Sicherungen | DB-08, DB-24, DB-74 |

### Beat im Verhalten (beobachtbar, in eigenen Worten)

- **Paginierung:** Ein Szenenkopf steht nie allein am Seitenende. Dialog wird nur an einer
  Klammer oder zwischen Sätzen geteilt, mit (MORE) unten und der Figur mit (CONT'D) oben.
  Bleiben weniger als zwei Zeilen auf der Seite, beginnt der Block auf der nächsten.
  Handlung wird zwischen Sätzen geteilt. Erzwungene Seitenumbrüche gelten.
- **Revisionen:** Jede Revisionsrunde hat eine Farbe und eine Randmarke (Blau `*`,
  Rosa `**`, Gelb `+`, Grün `++`, …); markiert werden hinzugefügter, gestrichener und zum
  Streichen vorgeschlagener Text.
- **Metadaten:** Beim Speichern hängt Beat Einstellungen, Revisionen, Tags, Figurendaten und
  gesperrte Nummern als JSON in einem Kommentarblock ans Ende der Fountain-Datei. Andere
  Programme zeigen ihn als auskommentierten Text; ein Diff in Git zeigt bei jeder Änderung den
  Block. Darum Frage F-02.
- **Tagging:** Feste Kategorien (Besetzung, Requisite, Spezialeffekt, Toneffekt, Kamera, Tier,
  Statisterie, Fahrzeug, Kostüm, Maske, Musik, Ton, Stunt, Sonstiges), je mit Farbe; daraus
  entsteht ein Bericht je Kategorie.
- **Statistik:** Zeilen und Wörter je Figur, Verteilung nach Geschlecht (vom Nutzer gesetzt),
  Szenen je Ort und Tageszeit.

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
| 4 | Drehbuchauszug und Berichte fließen ohne Export in Planung | DB-60, DB-61 |
| 5 | Kein Datenverlust bei Absturz, Sicherung wie Stellprobe | DN-06 |

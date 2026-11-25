# Oberfläche: Entscheidungen der Designrunde

Stand: 2026-11-19. Grundlage von D-01 bis D-10: Designrunde mit Bildern aller Ansichten auf main
7c820b6 (https://claude.ai/artifact/L2GFw9iK7vwjqHMXbfcyaG); von D-11 bis D-17: Entscheidungsrunde
Module mit Bildern von main e85289e4 (https://claude.ai/artifact/6wLaK6otgShSzdqkRcSk6Q); der Eigentümer hat je Ansicht entschieden.
Dieses Dokument hält die Entscheidungen und die Reihenfolge der Umsetzung fest; `STAND.md` führt,
was davon fertig ist. Es löst die Oberflächen-Entscheidung E-14 in `KONZEPT.md` ab, wo die
beiden sich unterscheiden (Werkzeugleiste, Shot-Bereich, Ebenen, Plan-Auswahl).

## Zielbild Hauptfenster

```text
┌────────────────────────────────────────────────────────────────────────────────┐
│ KLAPPE  Wartburg › Szene 12 › Plan A ▾   Datei ▾ Ausgeben ▾ Ansicht ▾  ↶ ↷ ⌕   │ Kopf
├──────────┬─────────────────────────────────────────────────────────────────────┤
│ DREHBUCH │ [Set bauen|Inszenieren|Auflösen|Zeichnen]          ⌗ Einrasten  ◧ │ Modus
│          │┌──┐                                              ┌──────────────┐  │
│ Text …   ││▣ │            Plan, randlos                     │ Shots        │  │
│          ││◯ │      (füllt den Bereich ganz)                │ Kacheln|Liste│  │
│          ││▭ │                  ┌──────────┐                │              │  │
│          ││… │                  │Karte am  │                └──────────────┘  │
│          │└──┘                  │Element   │             schwebende Paletten │
│          │ Werkzeuge des Modus  └──────────┘                                   │
├──────────┴─────────────────────────────────────────────────────────────────────┤
│ ⏮ ▶ ⏭  00:08,0 / 00:20,0   Schleife Shot                         Schnittfolge ▾│ Transport
│ Schnittfolge ███████ 12.1 ███ 12.3 ██████████ 12.4 ███████████ 12.5 ████        │ Zeitleiste
│ Elisabeth        ▓▓▓▓▓▓▓▓▓▓            ▓▓▓▓▓▓▓▓▓                                │
└────────────────────────────────────────────────────────────────────────────────┘
```

## Entscheidungen je Ansicht

| Nr | Ansicht | Entscheidung |
|---|---|---|
| D-01 | Fensteraufbau | Mischung: Werkzeugleiste als Symbol-Leiste (Gruppen, Tooltip mit Taste), Arbeitsmodi (Set bauen, Inszenieren, Auflösen, Zeichnen; Tasten 1–4) und Plan randlos mit schwebenden, einklappbaren Paletten. Das Drehbuch bleibt ein eigener Bereich am linken Rand (seit D-11 und D-14: Modulleiste ganz links, die Spalte mit Reitern Szenen | Text; das Zielbild oben zeigt den Stand davor). Werkzeuge je Modus siehe unten. |
| D-02 | Kopf und Werkzeuge | Befehlspalette (Strg+K: Werkzeuge, Exporte, Ansichten, Projektbefehle mit Suche) und geordnete Menüs im Kopf (Datei, Ausgeben, Ansicht); Rückgängig/Wiederholen als Symbole. Einrasten ist ein Schalter, kein Werkzeug. |
| D-03 | Plan-Varianten | Im Pfad: der Planname im Kopf ist ein Menü mit Liste der Varianten, Schnappschuss, Neu, Umbenennen, Löschen (abgesetzt). Die eigene Zeile über dem Plan entfällt. |
| D-04 | Karten der Auswahl | Am Element: die Karte (Objekt, Kamera, Mark, Notiz) öffnet neben dem gewählten Element, mit einem Pfeil zum Element, und weicht dem Mauszeiger aus; häufige Felder, „Mehr …“ für alles. Bei einer Kamera eine Karte mit Abschnitten statt zwei Karten. |
| D-05 | Shot-Bereich | Zwei Ansichten, umschaltbar: Storyboard-Kacheln (Bildformat der Kamera, Kamerablick des Shots zu seinem Beginn LH-36) und Shotliste als Tabelle mit Bearbeiten in der Zelle. Details des gewählten Shots in einer Karte bzw. Schublade. |
| D-06 | Zeitleiste und Ebenen | Transportleiste im Kopf der Zeitleiste (Abspielen, Sprung von Mark zu Mark, Zeit groß, Schleife Shot; LH-60). Ebenen in ein Popover am Plan (Liste mit Auge und Schloss; Punkt am Knopf, wenn etwas aus oder gesperrt ist). |
| D-07 | Startseite | Projektkacheln mit Grundriss-Vorschau der ersten Szene, Datum und Umfang; erste Kachel „Leerer Plan“ legt ohne Rückfrage ein Projekt mit einer Szene und leerem Plan an und öffnet es; letzte Kachel öffnet das Beispielprojekt. Kopf ohne Projektknöpfe, solange kein Projekt offen ist. |
| D-08 | Projekt verwalten | Aus dem Drehbuch heraus (Szenenkopf und Figurennamen anklickbar, Karte mit Eigenschaften und Vorkommen) und dazu ein zweispaltiger Dialog (Liste links, Formular rechts, kein Dialog im Dialog). Eine Änderung, die das Drehbuch ändert (Szenenkopf, Figurenname), zeigt vorher eine Warnung mit dem, was sich im Text ändert. |
| D-09 | Schmale Fenster | Feste Breitenstufen (≥ 1400 alles; 1000–1400 Shots und Paletten als Schubladen; < 1000 Plan und Zeitleiste, Rest über Reiter) und ziehbare Teiler (`KanteSplitHandle`, Breite gemerkt, Doppelklick klappt ein). Nichts wird abgeschnitten; Überlauf geht ins Menü. Je Stufe ein Screenshot-Test. |
| D-10 | TUI | Kante-Farben (aktive Szene gelb, Marken für Shots und Marks rechts an der Zeile, Szenenkopf ohne `#12#`, Sprache wie die App) und dichte Ansicht (Szenenwechsel über Strg+P mit Suche, ganze Breite für den Text). |
| D-11 | Modulwechsel | Modulleiste: schmale Symbolleiste am linken Rand, ein Symbol je Modul (Drehbuch, Stellprobe, Storyboard, Breakdown, Besetzung/Team, Drehplan, Dispo), Tooltip mit Taste. Die gewählte Szene bleibt beim Wechsel; der Pfad im Kopf zeigt, was das Modul in ihr offen hat. |
| D-12 | Tasten der Module | Strg+1 bis Strg+7 in der Reihenfolge der Leiste; Strg+E (heute Drehbuch-Ansicht in der App) entfällt dort; im TUI bleibt Strg+E „Drehbuch als PDF“. Die Tasten 1–4 ohne Strg bleiben die Modi der Stellprobe. |
| D-13 | Szenenliste | Gemeinsame Szenenleiste: alle Szenen des Projekts (nicht nur die Köpfe im Text), in jedem Modul gleich, mit Marken für Text, Plan, Shots; Szenen ohne Text als „nicht im Buch“. Ab 1400 px sichtbar, darunter Schublade (D-09). |
| D-14 | Linke Spalte der Stellprobe | Reiter „Szenen“ und „Text“ in derselben Spalte; „Text“ ist der heutige Drehbuchtext der Szene mit den Chips. |
| D-15 | Drehbuch-Modul | Fließtext über die ganze Breite, Elemente eingerückt, ohne Seiten; ein Schalter für das Druckbild (Seiten mit Umbrüchen wie gedruckt). |
| D-16 | Import eines Drehbuchs | Gleicht sofort ab: die Szenen entstehen beim Import, nicht erst beim Speichern; danach ein Bericht („14 Szenen erkannt · 3 neu · 2 Fragen“) mit Knopf zu den Fragen. |
| D-17 | Leere Flächen | Jede leere Fläche sagt, warum sie leer ist, und bietet den nächsten Schritt als Knopf (Szene ohne Plan: „Plan anlegen“; Spalte ohne Text: „Drehbuch laden …“ oder „Drehbuch öffnen“). Nichts entsteht ohne Klick. |
| D-18 | Suchen im Drehbuch | Strg+F öffnet eine schwebende Karte oben rechts über dem Text (Suchen, Treffer, ↑ ↓, ✕; darunter Ersetzen, Alle ersetzen, Groß-/Kleinschreibung, „Nur Dialog von“); sie verschiebt nichts und wandert nach unten rechts, solange der gezeigte Treffer unter ihr liegt (wieder nach oben, wenn ein späterer Treffer dort unter ihr liegt). Entschieden am 20.11.2026 (PR #288). |
| D-19 | Änderungen von außen | Das Fenster prüft beim Aktivieren und vor dem Speichern, ob sich Dateien des Projektordners außerhalb geändert haben. Nichts Ungespeichertes: es lädt still neu, ein Hinweis nennt die Dateien, Rückgängig ist leer. Mit Ungespeichertem: sofort ein Hinweis über dem Plan, Speichern fragt in einem Dialog (Neu laden / Überschreiben / Abbrechen). Nur eine Datei, die beide geändert haben, braucht eine Frage; Drehbuch und Plan getrennt gehen ohne. Das TUI fragt vor Strg+S in einem Kasten wie Strg+K. Entschieden am 22.11.2026. |
| D-20 | Weitere Namen und „Keine Figur“ (DB-50) | Die Figurenverwaltung zeigt die weiteren Namen einer Figur als Chips mit Kreuz, ein Feld darunter fügt einen hinzu (Enter; ein getippter Name zählt auch beim Übernehmen). Die Namen, die keine Figur sind (`ALLE`, `STIMME`), stehen unter der Liste der Figuren, ebenso als Chips mit Feld. Entfernt man einen Namen, den das Buch benutzt, warnt die Verwaltung erst: das Buch fragt dann wieder nach ihm, der Text bleibt. Die TUI bekommt keine Verwaltung dafür. Entschieden am 23.11.2026. |
| D-21 | Spielort und Drehort (DB-51) | Ein eigener Reiter „Spielorte“ im Projektdialog listet die Spielorte mit ihrem Drehort („noch kein Drehort“ fällt auf); das Formular zeigt Name, Schreibweisen im Kopf (Chips wie D-20) und den Drehort. Eine Szene weicht im Szenenformular ab: die Auswahl zeigt zuerst „Wie Spielort: …“, ein anderer Drehort gilt nur für diese Szene. Liegt der Plan auf einem anderen Set als die Szene gedreht wird, steht ein Hinweis über dem Plan mit „Neuer Plan auf …“ und „Szene in … drehen“. Eine Szene ohne Drehort zeigt eine leere Fläche (D-17) mit „Drehort wählen“ und „Neuer Drehort“. Die TUI zeigt den Drehort nicht. Entschieden am 23.11.2026. |
| D-22 | Breakdown je Szene (DB-60, Datenmodell Schritt 2) | Eigenes Modul „Breakdown“ in der Leiste (Strg+3): Szenenliste links, rechts das Szenenblatt (Spieltag, Vorstopp, geplante Drehdauer, Continuity, Hinweis für die Produktion, Figuren, Elemente je Kategorie). Elemente kommen über ein Feld je Kategorie mit Vorschlägen aus vorhandenen Elementen in die Szene; Markieren im Drehbuchtext folgt als eigener Schritt. Das Szenenblatt zeigt nur belegte Kategorien und „Kategorie hinzufügen“. Ein Umschalter „Szenen \| Elemente“ über der Liste zeigt alle Elemente nach Kategorie, rechts das Element (Name, Kategorie, Beschreibung, Status, Szenen). Status: offen, in Arbeit, bereit. Ein Set-Objekt, das auf ein Element zeigt, erscheint gestrichelt als Vorschlag mit „übernehmen“ und „ausschließen“. Figuren: die sprechenden aus dem Buch fest, stumme Figuren und Komparsen kommen hinzu. Die TUI zeigt den Breakdown nicht. Entschieden am 23.11.2026. |
| D-23 | Markieren im Text der Szene (DB-60, Datenmodell Schritt 2, nach D-22) | Markiert wird im Modul Breakdown: das Szenenblatt zeigt den Text der Szene (nur lesen). Nach dem Wählen von Wörtern erscheint ein Menü an der Auswahl: oben vorhandene Elemente mit ähnlichem Namen, darunter die Kategorien für ein neues; Pfeile und Enter. Markierter Text ist unterstrichen in der Farbe der Kategorie, dahinter ihr Kürzel (Farbe nie allein, DB-22). Fehlt die Stelle nach einer Änderung des Buchs, bleibt das Element in der Szene und sein Chip zeigt „Stelle nicht mehr im Text“; man markiert neu oder entfernt den Hinweis. Steht der Name eines vorhandenen Elements im Text einer Szene, erscheint es dort als Vorschlag wie die aus dem Set. Der Fountain-Text bleibt sauber (F-02); die Stelle steht als Zitat (Wörter und wievieltes Vorkommen) im Szenenblatt. Das Drehbuch-Modul und die TUI zeigen keine Markierungen. Entschieden am 23.11.2026. |
| D-24 | Berichte im Fenster (DB-61) | Im Menü „Ausgeben“ steht unter den Ausgaben der Abschnitt „Berichte“ mit den fünf Berichten (Szenen, Spielorte, Figuren je Szene, Drehbuchauszug, Kontaktliste); dieselben stehen in der Palette. Jeder öffnet den Dateidialog wie die Shotliste, ohne Papierwahl: Dateiname und Format (PDF, CSV, ODS, XLSX) wählt man dort, die Endung des Namens gilt vor dem Filter. Fehler (z. B. „Keine Szenen im Buch“) und die Meldung „Bericht gespeichert“ stehen in der Statuszeile wie bei den anderen Ausgaben. Die TUI bekommt keine Berichte. Entschieden am 24.11.2026. |
| D-26 | Besetzung und Team (Datenmodell Schritt 3, nach D-11, D-22) | Zwei Module in der Leiste: „Besetzung“ (Strg+4) und „Team“ (Strg+5), mit „Gehe zu: …“ in der Palette. Besetzung: links die Rollen (die Figuren des Projekts, Gruppen und Komparsen zuletzt, mit Größe), je Zeile Cast-Nummer, Name und die feste Person oder „n Vorschläge“; rechts das Rollenformular: Cast-Nummer (zeigt sie und ob „automatisch“ oder „festgelegt“, ein Feld für die Nummer von Hand, leer = automatisch, eine abgelehnte Nummer ist wie andere abgelehnte Felder markiert), „Nummern festschreiben“ und „Nummern freigeben“ (für alle Rollen), die vorgeschlagenen Personen als Chips (die feste mit „fest“ beschriftet, je ✓ zum Festlegen und ×; der Chip nimmt den Tastaturfokus: Entf streicht, Eingabe legt fest), ein Suchfeld mit vorhandenen Personen (Department Darsteller zuerst) und „Neu: <Name>“, das eine Person im Department Darsteller anlegt und vorschlägt, und die Zahl der Szenen der Rolle. Team: links die Personen nach Department (ohne Department zuletzt) mit Filter und „Neu …“; rechts das Personenformular: Name, Department, Funktion, Agentur, Telefon, Mail, Adresse (je einzeilig, Kante hat kein mehrzeiliges Feld), Übernehmen, Löschen … (fragt und nennt, dass Besetzungen und Verantwortlichkeiten mitgehen); Übernehmen ist ein Schritt (`EditPerson` mit Kontakt). Je Element im Elementformular ein Feld „Verantwortlich“ mit den Personen, die des passenden Departments zuerst (`persons_for`), und „keine“ (Übernehmen sendet es mit `EditElement`, ein Schritt). Die TUI zeigt nichts davon. Leere Zustände wie im Reiter Elemente. Entschieden am 25.11.2026. |

## Werkzeuge je Modus

| Modus (Taste) | Werkzeuge |
|---|---|
| Set bauen (1) | Auswahl, Wand, Objekt, Maßstab, Messen |
| Inszenieren (2) | Auswahl, Figur (Marks, Wege), Messen |
| Auflösen (3) | Auswahl, Kamera, Licht, Messen |
| Zeichnen (4) | Auswahl, Notiz, Linie, Pfeil, Rechteck, Ellipse, Fläche, Messen |

Die Buchstabentasten der Werkzeuge (V, W, O, F, C, L, M, D, N, I, A, B, E, P) gelten in jedem
Modus; eine Taste eines anderen Modus wechselt in dessen Modus. Auswahl und Messen gibt es in
allen Modi. Ausgewählt und bearbeitet werden kann jedes sichtbare, nicht gesperrte Element in
jedem Modus; der Modus bestimmt nur, welche Werkzeuge die Leiste zeigt.

## Bausteine aus Kante

Kante hat: `KanteIcon`, `KanteToolButton`, `KantePopover`, `KanteSplitHandle`,
`KanteTransportBar`, `KanteTile`, `KanteTabBar`, `KanteGlassPanel`, `KanteSegmented`.
Neu in Kante #178 (Kante 1.81, für dieses Zielbild):

- `KanteShapeCanvas`: Farbe `ground` (Grund des Plans), damit Zeichen auf gefüllten Formen
  lesbar sind (Punkte einer Gruppe, PR #39).
- `KantePopover.openAtPoint`: öffnen an einem Punkt statt an einem Item, mit Zeiger.
- `KanteCommandPalette`: Suchfeld mit Liste von Befehlen (Name, Gruppe, Taste), Strg+K.
- `KanteFloatingPanel`: Karte mit Kopf zum Einklappen über einem Inhalt (randloser Plan).
- `KanteSplitHandle.collapsible`: Einklappen per Doppelklick oder Taste (D-09).
- `KanteTile.stageAspect`: Seitenverhältnis der Bühne (D-05).
- `KanteTransportBar`: Sprung zum vorigen/nächsten Marker, Zeit mit Zehnteln (D-06).

Fehlt etwas Weiteres, kommt es zuerst in Kante (GUI-Regel).

## Reihenfolge

1. Kante-Bausteine (eigener PR in Kante), danach `scripts/sync-kante.sh`.
2. Fenster: Kopf mit Pfad, Menüs und Planmenü (D-02, D-03), Modi und Symbol-Leiste (D-01),
   Plan randlos mit Paletten, Teiler und Breitenstufen (D-09).
3. Karten am Element (D-04).
4. Transportleiste und Ebenen-Popover (D-06).
5. Shots als Kacheln und Tabelle (D-05).
6. Startseite (D-07).
7. Projekt aus dem Drehbuch und zweispaltiger Dialog (D-08).
8. TUI (D-10) – unabhängig, läuft parallel.
9. Import mit sofortigem Abgleich und Bericht (D-16), leere Flächen (D-17).
10. Gemeinsame Szenenleiste (D-13), zuerst in Drehbuch und Stellprobe, mit Reitern Szenen/Text (D-14).
11. Modulleiste und Strg+Ziffer (D-11, D-12); das Drehbuch wird das erste Modul neben der Stellprobe.
12. Drehbuch-Editor als Fließtext mit Elementen und Schalter für das Druckbild (D-15).

Jeder Schritt ist ein eigener PR und hält die bestehenden Funktionen; QML-Tests und Screenshots
der App (offscreen) zeigen den neuen Stand.

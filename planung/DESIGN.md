# Oberfläche: Entscheidungen der Designrunde

Stand: 2026-11-14. Grundlage: Designrunde mit Bildern aller Ansichten auf main 7c820b6
(https://claude.ai/artifact/L2GFw9iK7vwjqHMXbfcyaG); der Eigentümer hat je Ansicht entschieden.
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
| D-01 | Fensteraufbau | Mischung: Werkzeugleiste als Symbol-Leiste (Gruppen, Tooltip mit Taste), Arbeitsmodi (Set bauen, Inszenieren, Auflösen, Zeichnen; Tasten 1–4) und Plan randlos mit schwebenden, einklappbaren Paletten. Das Drehbuch bleibt ein eigener Bereich am linken Rand. Werkzeuge je Modus siehe unten. |
| D-02 | Kopf und Werkzeuge | Befehlspalette (Strg+K: Werkzeuge, Exporte, Ansichten, Projektbefehle mit Suche) und geordnete Menüs im Kopf (Datei, Ausgeben, Ansicht); Rückgängig/Wiederholen als Symbole. Einrasten ist ein Schalter, kein Werkzeug. |
| D-03 | Plan-Varianten | Im Pfad: der Planname im Kopf ist ein Menü mit Liste der Varianten, Schnappschuss, Neu, Umbenennen, Löschen (abgesetzt). Die eigene Zeile über dem Plan entfällt. |
| D-04 | Karten der Auswahl | Am Element: die Karte (Objekt, Kamera, Mark, Notiz) öffnet neben dem gewählten Element, mit einem Pfeil zum Element, und weicht dem Mauszeiger aus; häufige Felder, „Mehr …“ für alles. Bei einer Kamera eine Karte mit Abschnitten statt zwei Karten. |
| D-05 | Shot-Bereich | Zwei Ansichten, umschaltbar: Storyboard-Kacheln (Bildformat der Kamera, später Kamerablick LH-36) und Shotliste als Tabelle mit Bearbeiten in der Zelle. Details des gewählten Shots in einer Karte bzw. Schublade. |
| D-06 | Zeitleiste und Ebenen | Transportleiste im Kopf der Zeitleiste (Abspielen, Sprung von Mark zu Mark, Zeit groß, Schleife Shot; LH-60). Ebenen in ein Popover am Plan (Liste mit Auge und Schloss; Punkt am Knopf, wenn etwas aus oder gesperrt ist). |
| D-07 | Startseite | Projektkacheln mit Grundriss-Vorschau der ersten Szene, Datum und Umfang; eine Kachel öffnet das Beispielprojekt. Kopf ohne Projektknöpfe, solange kein Projekt offen ist. |
| D-08 | Projekt verwalten | Aus dem Drehbuch heraus (Szenenkopf und Figurennamen anklickbar, Karte mit Eigenschaften und Vorkommen) und dazu ein zweispaltiger Dialog (Liste links, Formular rechts, kein Dialog im Dialog). Eine Änderung, die das Drehbuch ändert (Szenenkopf, Figurenname), zeigt vorher eine Warnung mit dem, was sich im Text ändert. |
| D-09 | Schmale Fenster | Feste Breitenstufen (≥ 1400 alles; 1000–1400 Shots und Paletten als Schubladen; < 1000 Plan und Zeitleiste, Rest über Reiter) und ziehbare Teiler (`KanteSplitHandle`, Breite gemerkt, Doppelklick klappt ein). Nichts wird abgeschnitten; Überlauf geht ins Menü. Je Stufe ein Screenshot-Test. |
| D-10 | TUI | Kante-Farben (aktive Szene gelb, Marken für Shots und Marks rechts an der Zeile, Szenenkopf ohne `#12#`, Sprache wie die App) und dichte Ansicht (Szenenwechsel über Strg+P mit Suche, ganze Breite für den Text). |

## Werkzeuge je Modus

| Modus (Taste) | Werkzeuge |
|---|---|
| Set bauen (1) | Auswahl, Wand, Objekt, Maßstab, Messen |
| Inszenieren (2) | Auswahl, Figur (Marks, Wege), Messen |
| Auflösen (3) | Auswahl, Kamera, Licht, Messen |
| Zeichnen (4) | Auswahl, Notiz, Linie, Pfeil, Rechteck, Ellipse, Fläche |

Die Buchstabentasten der Werkzeuge (V, W, O, F, C, L, M, D, N, I, A, B, E, P) gelten in jedem
Modus; eine Taste eines anderen Modus wechselt in dessen Modus. Auswahl und Messen gibt es in
allen Modi. Ausgewählt und bearbeitet werden kann jedes sichtbare, nicht gesperrte Element in
jedem Modus; der Modus bestimmt nur, welche Werkzeuge die Leiste zeigt.

## Bausteine aus Kante

Kante hat: `KanteIcon`, `KanteToolButton`, `KantePopover`, `KanteSplitHandle`,
`KanteTransportBar`, `KanteTile`, `KanteTabBar`, `KanteGlassPanel`, `KanteSegmented`.
Fehlt und kommt zuerst in Kante (GUI-Regel):

- `KanteShapeCanvas`: Farbe `ground` (Grund des Plans), damit Zeichen auf gefüllten Formen
  lesbar sind (Punkte einer Gruppe, PR #39).
- `KantePopover`: öffnen an einem Punkt (Szenenkoordinaten) statt an einem Item, mit Zeiger.
- Befehlspalette: Suchfeld mit Liste von Befehlen (Name, Gruppe, Taste), Tastatur, Strg+K.
- Schwebende Palette: Karte mit Kopf zum Einklappen über einem Inhalt (für den randlosen Plan).
- `KanteSplitHandle`: Einklappen per Doppelklick (heute setzt er auf `defaultValue` zurück; D-09).
- `KanteTile`: Seitenverhältnis der Bühne einstellbar (heute fest 3:2; D-05 braucht 2,39:1 usw.).
- `KanteTransportBar`: Sprung zum vorigen/nächsten Marker und Zeit mit Zehnteln (D-06).

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

Jeder Schritt ist ein eigener PR und hält die bestehenden Funktionen; QML-Tests und Screenshots
der App (offscreen) zeigen den neuen Stand.

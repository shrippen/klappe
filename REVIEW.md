# Review guide

Every PR is reviewed before it is merged (rule in `RULES.md`, procedure in the skill
`pr-review`): by a new Claude agent of the type `reviewer`. The reviewer knows nothing of the
session that wrote the change: it gets the PR's intent, its diff and the repository, and this
file. The owner does not read code; this review is the second reader.

## What failure looks like here

Klappe is a film preproduction suite; its first module, Stellprobe, plans floor plans:
figures and cameras moving over time, angles of view, the 180° axis, a shot list. A bug that
matters is one a director plans wrongly with, sees wrongly or loses work to:

- **Plans wrongly:** a wrong angle of view (sensor width, focal length), a figure or camera
  at the wrong place or time, an axis jump missed or reported where there is none, units
  that do not match their names (`_m`, `_deg`, `_s`, `_mm`), angles in the wrong direction
  (0° = right, clockwise on screen, y down).
- **Loses work:** a project that does not load again as it was saved, a format change
  without `FORMAT_VERSION` and migration once a release exists.
- **Breaks the layers:** a crate calling past its neighbour below (`agent.md`), geometry
  computed in QML instead of the display list, `unsafe` outside the cxx-qt bridge,
  `unwrap`/`expect` outside tests, a new `pub` not named in the PR.
- **Looks broken:** own colours or sizes instead of Kante roles, an edit in the copied
  `crates/app/qml/Kante`, information shown only by colour, a text in code instead of a
  translation key (de and en).
- **Brings in what it must not:** a dependency without a reason in the PR or under a licence
  `agent.md` forbids (GPL, MPL, LGPL crates); anything taken from Shot Designer (clean room,
  `planung/stellprobe/LASTENHEFT.md`, Anhang A).
- **Lies:** `planung/`, README, `agent.md` or comments that the change made wrong.

## What to check in particular

- New behaviour has a test; geometry and time have tests with numbers worked out by hand.
- Bug fixes: does the fix remove the cause, with a test that failed before?
- `RULES.md` code rules: braces always, early return, enums not bools, short function
  names, constants instead of magic numbers.

## What the owner confirms here

"What is published" (rule in `RULES.md`) means here: a release (tag, release notes, a
landing page), packages (`packaging/`, a Flatpak workflow).

## Skip

- `crates/app/qml/Kante/` (copied by `scripts/sync-kante.sh`; check only that it is
  unchanged), `Cargo.lock`, screenshots and icons.

## How to report

- Every finding names file and line and the concrete input or state that goes wrong. No
  guess without evidence; if unsure, say so.
- Each finding gets one label: **Wichtig** (a bug from the list above, must be fixed or
  answered before the merge), **Kleinigkeit** (optional, at most five) or **Bestand** (the
  bug was there before this PR; not blocking).
- The report starts with the count, e.g. "2 Wichtig, 1 Kleinigkeit" (the session puts the
  heading above it). The count of **Wichtig** holds every one still open, also from an
  earlier round. Only when none is open it says plainly: "Nichts zu beheben."
- In a later round: say for each open **Wichtig** of the last round whether it is fixed, or
  whether its answer is accepted, with the reason.
- Findings in German, code names as they are.

.PHONY: check fmt structure lint test qml perf deny advisories commits run kante

# Everything the CI's core job runs; run before every commit (agent.md).
check: fmt structure lint test qml deny commits

fmt:
	cargo fmt --all --check

# Function and file sizes within the limits or structure.baseline (RULES.md); needs pipx.
structure:
	pipx run tools/structure-check.py

lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Also checks that demo/beispiel-*.klappe are what `cargo run -p klappe-demo` makes of demo/world.json.
test:
	cargo test --workspace

# The app's QML tests, offscreen; skipped with a note without qmltestrunner (qt6-declarative).
qml:
	scripts/check-qml.sh

# NF-05 timings on a large plan (50 figures, 10 cameras, 300 objects): release build, prints the
# numbers, fails above four times the 16 ms budget (the plan JSON: four times its half); the
# screenplay's 120 pages within 1 s (DN-02); matching a 120-page script against its previous
# state (`align_perf`) within 100 ms. Not in `check`: timings need a quiet machine.
perf:
	cargo test --release -p klappe-services --test perf -- --ignored --nocapture
	cargo test --release -p klappe-script --test align_perf -- --ignored --nocapture
	cargo test --release -p klappe-app a_large_plan -- --ignored --nocapture
	cargo test --release -p klappe-script --test perf -- --ignored --nocapture
	scripts/check-qml.sh crates/app/tests/perf

# Licence policy (deny.toml, E-05); `cargo install --locked cargo-deny` once.
deny:
	cargo deny check licenses bans sources

# Known vulnerabilities (needs the network).
advisories:
	cargo deny check advisories

# Commit messages of this branch (RULES.md, commit rules 1-6).
commits:
	python3 scripts/commit-msg.py --range origin/main..HEAD

# The app on the example project (needs Qt 6 and Kirigami).
run:
	cargo run -p klappe-app -- --open testdata/beispiel.klappe

# Copy Kante (QML module, palette) unchanged; KANTE_DS=<checkout> if not ../Kante.
kante:
	scripts/sync-kante.sh

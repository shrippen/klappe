.PHONY: check fmt lint test qml deny advisories commits run kante

# Everything the CI's core job runs; run before every commit (agent.md).
check: fmt lint test qml deny commits

fmt:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Also checks that demo/beispiel-*.klappe are what `cargo run -p klappe-demo` makes of demo/world.json.
test:
	cargo test --workspace

# The app's QML tests, offscreen; skipped with a note without qmltestrunner (qt6-declarative).
qml:
	scripts/check-qml.sh

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

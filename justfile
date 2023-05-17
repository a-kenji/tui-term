alias d := doc
alias l := lint
alias uf := update-flake-dependencies
alias uc := update-cargo-dependencies
alias r := run
alias t := cargo-test
alias b := build
alias rr := run-release
alias cw := cargo-watch

clippy:
	cargo clippy --all-targets --all-features
actionlint:
	nix develop .#actionlintShell --command actionlint
deny:
	cargo deny check
cargo-test:
	cargo test
lint:
	nix flake check
	typos
run:
	cargo run
build:
	cargo build
run-release:
	cargo run --release

doc:
    cargo doc --open --offline

# Update and then commit the `Cargo.lock` file
update-cargo-dependencies:
    cargo update
    git add Cargo.lock
    git commit Cargo.lock -m "update(cargo): \`Cargo.lock\`"

# Future incompatibility report, run regularly
cargo-future:
    cargo check --future-incompat-report

update-flake-dependencies:
    nix flake update --commit-lock-file

cargo-watch:
    cargo watch -x check -x test -x build

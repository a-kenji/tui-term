# Releases

Releases will attempt to track upstream `ratatui` releases.

## Creating a Release

1. Adjust `docs/assets/demo.tape`, and regenerate if necessary.
1. Update the changelog.
1. Run the release script: `./docs/assets/bin/create-release.sh <version>`
1. Publish to crates.io: `cargo publish`

The script handles version bumping (Cargo.toml, README.md, Cargo.lock),
PR creation, merging into the release branch, tagging, and drafting a
GitHub release.
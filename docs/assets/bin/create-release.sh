#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null && pwd)"
cd "$SCRIPT_DIR/../../.."

version="${1:-}"
if [[ -z $version ]]; then
  echo "USAGE: $0 version" >&2
  echo "" >&2
  echo "Before running this script:" >&2
  echo "  1. Update CHANGELOG.md (git cliff -o CHANGELOG.md, then adjust)" >&2
  echo "  2. Update docs/assets/demo.tape and regenerate if necessary" >&2
  exit 1
fi

version="${version#v}"

if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "invalid version '${version}', expected semver (e.g. 0.4.0)" >&2
  exit 1
fi

if [[ "$(git symbolic-ref --short HEAD)" != "development" ]]; then
  echo "must be on development branch" >&2
  exit 1
fi

waitForPr() {
  local pr=$1
  while true; do
    state=$(gh pr view "$pr" --json state -q .state)
    if [[ $state == "MERGED" ]]; then
      break
    fi
    echo "Waiting for PR to be merged..."
    sleep 5
  done
}

uncommitted_changes=$(git diff --compact-summary)
if [[ -n $uncommitted_changes ]]; then
  echo -e "There are uncommitted changes, exiting:\n${uncommitted_changes}" >&2
  exit 1
fi
git pull git@github.com:a-kenji/tui-term development
unpushed_commits=$(git log --format=oneline origin/development..development)
if [[ $unpushed_commits != "" ]]; then
  echo -e "\nThere are unpushed changes, exiting:\n$unpushed_commits" >&2
  exit 1
fi

if git tag -l | grep -q "^v${version}\$"; then
  echo "Tag v${version} already exists, exiting" >&2
  exit 1
fi

if ! grep -q "## \[${version}\]" CHANGELOG.md; then
  echo "CHANGELOG.md does not contain a section for [${version}]." >&2
  echo "Update the changelog before running this script:" >&2
  echo "  git cliff -o CHANGELOG.md  # then adjust manually" >&2
  exit 1
fi

sed -i -e "s/^version = \".*\"/version = \"${version}\"/" Cargo.toml
sed -i -e "s/tui-term = \"[0-9]*\.[0-9]*\.[0-9]*\"/tui-term = \"${version}\"/" README.md

cargo check

git add Cargo.toml Cargo.lock README.md CHANGELOG.md
git branch -D "release-${version}" 2>/dev/null || true
git checkout -b "release-${version}"
git commit -m "chore: release tui-term ${version}"
git push origin "release-${version}"
pr_url=$(gh pr create \
  --base development \
  --head "release-${version}" \
  --title "Release ${version}" \
  --body "Release ${version} of tui-term")

pr_number=$(echo "$pr_url" | grep -oE '[0-9]+$')

gh pr merge "$pr_number" --auto --rebase --delete-branch
git checkout development

waitForPr "release-${version}"
git pull git@github.com:a-kenji/tui-term development

git checkout release
git pull git@github.com:a-kenji/tui-term release
git merge development
git push origin release

git tag "v${version}"
git push origin "v${version}"

gh release create "v${version}" --draft --title "v${version}" --generate-notes

echo ""
echo "Release v${version} tagged and GitHub release drafted."
echo "To publish to crates.io:"
echo "  cargo publish --dry-run"
echo "  cargo publish"

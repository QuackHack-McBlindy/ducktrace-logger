#!/usr/bin/env bash
set -e

current_version=$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo "Current version: $current_version"

IFS='.' read -r major minor patch <<< "$current_version"
new_patch=$((patch + 1))
new_version="$major.$minor.$new_patch"
echo "New version: $new_version"

# Update Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
rm Cargo.toml.bak

# Update version in README.md
sed -i.bak "s/ducktrace-logger = \".*\"/ducktrace-logger = \"$new_version\"/" README.md
rm README.md.bak


git add Cargo.toml README.md
git commit -m "Bump version to $new_version"
git tag "v$new_version"

# Push to origin (main branch)
git push origin main --tags

echo "Released version $new_version"

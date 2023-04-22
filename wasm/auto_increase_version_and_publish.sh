#!/bin/bash

# Read the current version from Cargo.toml
current_version=$(grep -E '^version' Cargo.toml | sed 's/^version = "\(.*\)"/\1/')

# Split the version string into major, minor, and patch components
IFS='.' read -ra version_parts <<< "$current_version"

# Increment the patch version by 1
new_patch_version=$((version_parts[2] + 1))

# Reconstruct the new version string
new_version="${version_parts[0]}.${version_parts[1]}.$new_patch_version"

# Replace the old version with the new version in Cargo.toml
sed -i "s/^version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml

# Print the new version
echo "Updated version: $new_version"

wasm-pack build -t web -s positionex

wasm-pack publish -t web

echo "Done!!"

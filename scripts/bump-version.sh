#!/usr/bin/env bash
#
# Bump version across all project manifests.
#
# Usage:
#   ./scripts/bump-version.sh <version|major|minor|patch> [--dry-run]
#
# Examples:
#   ./scripts/bump-version.sh 0.2.0
#   ./scripts/bump-version.sh patch
#   ./scripts/bump-version.sh minor --dry-run

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DRY_RUN=false
VERSION_ARG=""

# Parse arguments
for arg in "$@"; do
    case $arg in
        --dry-run)
            DRY_RUN=true
            ;;
        *)
            VERSION_ARG="$arg"
            ;;
    esac
done

if [[ -z "$VERSION_ARG" ]]; then
    echo "Usage: $0 <version|major|minor|patch> [--dry-run]"
    echo ""
    echo "Arguments:"
    echo "  version    Explicit version (e.g., 0.2.0) or bump type (major, minor, patch)"
    echo "  --dry-run  Show what would change without modifying files"
    exit 1
fi

# Get current version from workspace Cargo.toml
get_current_version() {
    grep -m1 '^version = ' "$PROJECT_ROOT/Cargo.toml" | cut -d'"' -f2
}

# Calculate new version based on bump type
calculate_version() {
    local current="$1"
    local bump_type="$2"

    IFS='.' read -r major minor patch <<< "$current"

    case $bump_type in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            # Assume explicit version
            echo "$bump_type"
            ;;
    esac
}

# Validate semver format
validate_semver() {
    local version="$1"
    if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?(\+[a-zA-Z0-9.-]+)?$ ]]; then
        echo "Error: Invalid semver format: $version"
        exit 1
    fi
}

# Update version in Cargo.toml (workspace)
update_cargo_toml() {
    local file="$1"
    local version="$2"

    if $DRY_RUN; then
        echo "[dry-run] Would update $file"
        return
    fi

    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/^version = \"[^\"]*\"/version = \"$version\"/" "$file"
    else
        sed -i "s/^version = \"[^\"]*\"/version = \"$version\"/" "$file"
    fi
    echo "Updated: $file"
}

# Update version in package.json using jq or sed
update_package_json() {
    local file="$1"
    local version="$2"

    if [[ ! -f "$file" ]]; then
        echo "Warning: $file not found, skipping"
        return
    fi

    if $DRY_RUN; then
        echo "[dry-run] Would update $file"
        return
    fi

    if command -v jq &> /dev/null; then
        local tmp
        tmp=$(mktemp)
        jq --arg v "$version" '.version = $v' "$file" > "$tmp" && mv "$tmp" "$file"
    else
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "s/\"version\": \"[^\"]*\"/\"version\": \"$version\"/" "$file"
        else
            sed -i "s/\"version\": \"[^\"]*\"/\"version\": \"$version\"/" "$file"
        fi
    fi
    echo "Updated: $file"
}

# Update optional dependencies version in package.json
update_package_json_optional_deps() {
    local file="$1"
    local version="$2"

    if [[ ! -f "$file" ]]; then
        return
    fi

    if $DRY_RUN; then
        echo "[dry-run] Would update optionalDependencies in $file"
        return
    fi

    if command -v jq &> /dev/null; then
        local tmp
        tmp=$(mktemp)
        jq --arg v "$version" '
            if .optionalDependencies then
                .optionalDependencies |= with_entries(.value = $v)
            else
                .
            end
        ' "$file" > "$tmp" && mv "$tmp" "$file"
        echo "Updated optionalDependencies in: $file"
    else
        echo "Warning: jq not found, cannot update optionalDependencies in $file"
    fi
}

# Update scrape-core and scrape-cli versions in workspace dependencies
update_workspace_deps() {
    local file="$1"
    local version="$2"

    if $DRY_RUN; then
        echo "[dry-run] Would update scrape-core and scrape-cli deps in $file"
        return
    fi

    if [[ "$(uname)" == "Darwin" ]]; then
        sed -i '' "s/\(scrape-core.*version = \"\)[^\"]*/\1$version/" "$file"
        sed -i '' "s/\(scrape-cli.*version = \"\)[^\"]*/\1$version/" "$file"
    else
        sed -i "s/\(scrape-core.*version = \"\)[^\"]*/\1$version/" "$file"
        sed -i "s/\(scrape-cli.*version = \"\)[^\"]*/\1$version/" "$file"
    fi
    echo "Updated scrape-core and scrape-cli dependencies in: $file"
}

main() {
    local current_version
    current_version=$(get_current_version)
    echo "Current version: $current_version"

    local new_version
    new_version=$(calculate_version "$current_version" "$VERSION_ARG")
    validate_semver "$new_version"
    echo "New version: $new_version"
    echo ""

    if $DRY_RUN; then
        echo "=== Dry Run Mode ==="
        echo ""
    fi

    # Update workspace Cargo.toml
    update_cargo_toml "$PROJECT_ROOT/Cargo.toml" "$new_version"
    update_workspace_deps "$PROJECT_ROOT/Cargo.toml" "$new_version"

    # Update Node.js package.json
    update_package_json "$PROJECT_ROOT/crates/scrape-node/package.json" "$new_version"
    update_package_json_optional_deps "$PROJECT_ROOT/crates/scrape-node/package.json" "$new_version"

    # Update WASM package.json
    update_package_json "$PROJECT_ROOT/crates/scrape-wasm/package.json" "$new_version"

    # Update npm platform packages
    local npm_dir="$PROJECT_ROOT/crates/scrape-node/npm"
    if [[ -d "$npm_dir" ]]; then
        for pkg_dir in "$npm_dir"/*/; do
            if [[ -f "$pkg_dir/package.json" ]]; then
                update_package_json "$pkg_dir/package.json" "$new_version"
            fi
        done
    fi

    echo ""
    echo "=== Version bump complete ==="
    echo ""
    echo "Next steps:"
    echo "  1. Review changes: git diff"
    echo "  2. Update CHANGELOG.md"
    echo "  3. Commit: git commit -am 'chore(release): $new_version'"
    echo "  4. Tag: git tag v$new_version"
    echo "  5. Push: git push && git push --tags"
}

main

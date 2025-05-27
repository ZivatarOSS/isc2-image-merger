#!/bin/bash

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting GitHub release process...${NC}"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed. Please install it first.${NC}"
    echo "Install with: brew install gh"
    exit 1
fi

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository.${NC}"
    exit 1
fi

# Check if user is authenticated with gh
if ! gh auth status &> /dev/null; then
    echo -e "${RED}Error: Not authenticated with GitHub CLI. Please run 'gh auth login' first.${NC}"
    exit 1
fi

# Read version from Cargo.toml
if [ ! -f "picmrg/Cargo.toml" ]; then
    echo -e "${RED}Error: picmrg/Cargo.toml not found.${NC}"
    exit 1
fi

VERSION=$(grep '^version = ' picmrg/Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: Could not extract version from picmrg/Cargo.toml${NC}"
    exit 1
fi

echo -e "${GREEN}Found version: ${VERSION}${NC}"

# Check if tag already exists
if git tag -l | grep -q "^v${VERSION}$"; then
    echo -e "${YELLOW}Warning: Tag v${VERSION} already exists. This will update the existing release.${NC}"
fi

# Define expected executables and their paths
EXECUTABLE_PATHS=(
    "picmrg/target/release/picmrg"
    "picmrg/target/x86_64-pc-windows-gnu/release/picmrg.exe"
    "picmrg/target/x86_64-unknown-linux-musl/release/picmrg"
    "picmrg/target/aarch64-unknown-linux-musl/release/picmrg"
    "picmrg/target/armv7-unknown-linux-musleabihf/release/picmrg"
)

RELEASE_NAMES=(
    "picmrg-macos"
    "picmrg-windows.exe"
    "picmrg-linux-x64"
    "picmrg-linux-arm64"
    "picmrg-linux-armv7"
)

# Check if all executables exist
echo -e "${GREEN}Checking for required executables...${NC}"
MISSING_FILES=()

for i in "${!EXECUTABLE_PATHS[@]}"; do
    file_path="${EXECUTABLE_PATHS[$i]}"
    release_name="${RELEASE_NAMES[$i]}"
    if [ ! -f "$file_path" ]; then
        MISSING_FILES+=("$file_path")
        echo -e "${RED}✗ Missing: $file_path (for $release_name)${NC}"
    else
        echo -e "${GREEN}✓ Found: $file_path (for $release_name)${NC}"
    fi
done

# Exit if any files are missing
if [ ${#MISSING_FILES[@]} -ne 0 ]; then
    echo -e "${RED}Error: Missing ${#MISSING_FILES[@]} required executable(s).${NC}"
    echo -e "${YELLOW}Please run ./build-all-targets.sh first to build all targets.${NC}"
    exit 1
fi

echo -e "${GREEN}All executables found!${NC}"

# Create or update the git tag
echo -e "${GREEN}Creating/updating git tag v${VERSION}...${NC}"
git tag -f "v${VERSION}"
git push origin "v${VERSION}" --force

# Create release notes
RELEASE_NOTES="Release v${VERSION}

This release includes pre-built binaries for multiple platforms:

- **macOS** (Intel/Apple Silicon): \`picmrg-macos\`
- **Windows** (64-bit): \`picmrg-windows.exe\`
- **Linux** (x86_64): \`picmrg-linux-x64\`
- **Linux** (ARM64): \`picmrg-linux-arm64\`
- **Linux** (ARMv7/Raspberry Pi): \`picmrg-linux-armv7\`

## Installation

Download the appropriate binary for your platform and make it executable:

\`\`\`bash
# For Linux/macOS
chmod +x picmrg-*
\`\`\`

## Usage

Run the binary directly or add it to your PATH for system-wide access."

# Create the release
echo -e "${GREEN}Creating GitHub release v${VERSION}...${NC}"
gh release create "v${VERSION}" \
    --title "Release v${VERSION}" \
    --notes "$RELEASE_NOTES" \
    --latest

# Upload all executables
echo -e "${GREEN}Uploading executables...${NC}"
for i in "${!EXECUTABLE_PATHS[@]}"; do
    file_path="${EXECUTABLE_PATHS[$i]}"
    release_name="${RELEASE_NAMES[$i]}"
    echo -e "${GREEN}Uploading $release_name...${NC}"
    gh release upload "v${VERSION}" "$file_path#$release_name" --clobber
done

echo -e "${GREEN}✅ Release v${VERSION} created successfully!${NC}"
echo -e "${GREEN}🔗 View the release at: $(gh repo view --web --json url -q .url)/releases/tag/v${VERSION}${NC}" 
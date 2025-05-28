#!/bin/bash
# Test Release Script
# This script helps you test what would be released from the current branch

set -e

echo "🔍 Testing semantic-release for current branch..."
echo ""

# Get current branch name
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
echo "📍 Current branch: $CURRENT_BRANCH"

# Check if there are uncommitted changes
if [[ -n $(git status --porcelain) ]]; then
    echo "⚠️  Warning: You have uncommitted changes. Consider committing them first."
    echo ""
fi

# Show recent commits
echo "📝 Recent commits:"
git log --oneline -5
echo ""

# Run semantic-release dry run
echo "🧪 Running semantic-release dry run..."
echo ""

npm run semantic-release:dry-run

echo ""
echo "✅ Test complete!"
echo ""
echo "💡 Tips:"
echo "  - If no release is shown, your commits may not trigger a release"
echo "  - Use conventional commit format: feat:, fix:, docs:, etc."
echo "  - Breaking changes need 'BREAKING CHANGE:' in footer or '!' after type"
echo "  - Feature branches will create pre-releases like: 1.0.0-feature-name.1"

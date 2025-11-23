# Pull Requests

This guide covers the pull request process for contributing to BotServer, from creation to merge.

## Overview

Pull requests (PRs) are the primary method for contributing code to BotServer. They provide code review, discussion, and testing before changes are merged.

## Before Creating a PR

### 1. Check Existing Work

- Search existing PRs to avoid duplication
- Check issues for related discussions
- Discuss major changes in an issue first

### 2. Prepare Your Branch

```bash
# Create feature branch from main
git checkout -b feature/your-feature

# Keep branch updated
git fetch origin
git rebase origin/main
```

### 3. Make Your Changes

- Follow [Code Standards](./standards.md)
- Write tests for new functionality
- Update documentation
- Keep commits atomic and logical

## Creating a Pull Request

### PR Title

Use clear, descriptive titles:

**Good Examples:**
- `feat: Add email notification support`
- `fix: Resolve session timeout issue`
- `docs: Update BASIC keyword reference`
- `refactor: Simplify session management`

**Bad Examples:**
- `Fix bug`
- `Update code`
- `WIP`

### PR Description Template

```markdown
## Description
Brief description of what this PR does.

## Type of Change
- [ ] Bug fix (non-breaking change)
- [ ] New feature (non-breaking change)
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring

## Changes Made
- List specific changes
- Include technical details
- Mention any side effects

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed
- [ ] No regression in existing features

## Documentation
- [ ] Code comments added/updated
- [ ] README updated if needed
- [ ] API docs updated if applicable
- [ ] CHANGELOG entry added

## Related Issues
Fixes #123
Relates to #456

## Screenshots (if applicable)
Add screenshots for UI changes

## Additional Context
Any other relevant information
```

## PR Best Practices

### Keep It Small

- Focus on one feature or fix
- Aim for < 500 lines changed
- Split large changes into multiple PRs
- Makes review easier and faster

### Commit Organization

```bash
# Good: Logical, atomic commits
git commit -m "feat: Add user session validation"
git commit -m "test: Add session validation tests"
git commit -m "docs: Update session API documentation"

# Bad: Mixed changes in one commit
git commit -m "Add feature and fix bugs and update docs"
```

### Self-Review First

Before requesting review:
1. Review your own changes
2. Check for debug code
3. Verify no accidental changes
4. Ensure consistent formatting
5. Test edge cases

## Code Review Process

### Requesting Review

1. Mark PR as ready (not draft)
2. Request specific reviewers if needed
3. Add relevant labels
4. Link related issues
5. Comment on complex areas

### Responding to Feedback

```markdown
# Acknowledge feedback
> "Thanks for catching that! Fixed in abc123"

# Explain decisions
> "I chose this approach because..."

# Ask for clarification
> "Could you elaborate on the performance concern?"

# Disagree respectfully
> "I see your point, but consider that..."
```

### Making Changes

```bash
# Address review comments
git commit -m "fix: Address review feedback"

# Or amend if appropriate
git commit --amend

# Force push to update PR
git push --force-with-lease origin feature/your-feature
```

## Review Guidelines

### For Reviewers

**Look For:**
- Code correctness
- Test coverage
- Documentation updates
- Performance implications
- Security considerations
- Code style consistency

**Provide:**
- Constructive feedback
- Specific suggestions
- Code examples when helpful
- Recognition of good work

### Review Comments

**Good Feedback:**
```rust
// Suggestion: Consider using `unwrap_or_default()` here
let value = option.unwrap_or_else(|| String::new());
// Could be:
let value = option.unwrap_or_default();
```

**Poor Feedback:**
```
This is wrong.
```

## CI/CD Checks

### Required Checks

All PRs must pass:
- `cargo build` - Compilation
- `cargo test` - Unit tests
- `cargo fmt -- --check` - Formatting
- `cargo clippy` - Linting
- Documentation build

### Fixing Failed Checks

```bash
# Format code
cargo fmt

# Fix clippy warnings
cargo clippy --fix

# Run tests locally
cargo test

# Check specific test
cargo test test_name -- --nocapture
```

## Merge Process

### Merge Requirements

- All CI checks pass
- At least one approval
- No unresolved conversations
- Branch up-to-date with main
- No merge conflicts

### Merge Methods

**Squash and Merge** (Preferred)
- Combines all commits
- Keeps history clean
- Good for feature PRs

**Rebase and Merge**
- Preserves commit history
- Good for well-organized PRs

**Create Merge Commit**
- Rarely used
- Only for special cases

## After Merge

### Clean Up

```bash
# Delete local branch
git branch -d feature/your-feature

# Delete remote branch (automatic on GitHub)
git push origin --delete feature/your-feature

# Update local main
git checkout main
git pull origin main
```

### Follow Up

- Monitor for issues
- Respond to questions
- Update related documentation
- Close related issues

## Common Issues

### Merge Conflicts

```bash
# Update branch with latest main
git fetch origin
git rebase origin/main

# Resolve conflicts
git status  # See conflicted files
# Edit files to resolve
git add .
git rebase --continue

# Or abort if needed
git rebase --abort
```

### Large PR

If PR becomes too large:
1. Close with explanation
2. Split into smaller PRs
3. Create tracking issue
4. Link all related PRs

### Stale PR

If PR goes stale:
- Ping reviewers
- Rebase on latest main
- Add comment with status
- Consider closing if abandoned

## Tips for Success

1. **Communicate Early**: Discuss before implementing
2. **Test Thoroughly**: Don't rely only on CI
3. **Be Patient**: Reviews take time
4. **Be Responsive**: Address feedback promptly
5. **Learn from Reviews**: Improve based on feedback
6. **Help Others**: Review other PRs too

## PR Labels

Common labels used:
- `breaking-change` - Requires major version bump
- `bug` - Bug fix
- `enhancement` - New feature
- `documentation` - Docs only
- `good-first-issue` - Good for newcomers
- `help-wanted` - Needs assistance
- `wip` - Work in progress

## Summary

Successful pull requests are:
- Well-prepared with clear purpose
- Properly documented and tested
- Responsive to feedback
- Focused and reasonably sized

Following these guidelines helps maintain code quality and makes the review process smooth for everyone involved.
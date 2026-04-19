# Pull Requests

This guide covers the pull request process for contributing to botserver, from creation to merge. Understanding this process helps ensure your contributions are reviewed efficiently and merged successfully.


## Overview

Pull requests are the primary method for contributing code to botserver. They provide a structured way to propose changes, enabling code review, discussion, and automated testing before changes are merged into the main codebase. Every contribution, whether a bug fix, new feature, or documentation update, follows this process.


## Before Creating a PR

### Check Existing Work

Before starting work on a contribution, search existing pull requests to avoid duplicating effort. Check the issue tracker for related discussions that might affect your approach. For major changes, open an issue first to discuss the design with maintainers and get feedback before investing significant time in implementation.

### Prepare Your Branch

Create a feature branch from the main branch for your work. Keep your branch updated by regularly fetching from origin and rebasing on the latest main. This practice reduces merge conflicts and ensures your changes work with the most recent codebase.

```bash
git checkout -b feature/your-feature
git fetch origin
git rebase origin/main
```

### Make Your Changes

Follow the established code standards documented in the standards guide. Write tests for any new functionality you add. Update documentation to reflect your changes. Keep commits atomic and logical, with each commit representing a single coherent change.


## Creating a Pull Request

### PR Title

Use clear, descriptive titles that follow the conventional commit format. Good titles include prefixes like "feat:" for new features, "fix:" for bug fixes, "docs:" for documentation updates, and "refactor:" for code restructuring. Examples of good titles include "feat: Add email notification support" and "fix: Resolve session timeout issue". Avoid vague titles like "Fix bug" or "Update code" that do not convey what the PR actually does.

### PR Description

The description should explain what the PR does and why. Start with a brief description of the change. Indicate the type of change whether it is a bug fix, new feature, breaking change, documentation update, performance improvement, or refactoring. List specific changes made with technical details and any side effects. Document testing performed including unit tests, integration tests, and manual testing. Note any documentation updates made. Link related issues using keywords like "Fixes #123" to automatically close issues when the PR merges. Include screenshots for UI changes.


## PR Best Practices

### Keep It Small

Focus each PR on one feature or fix rather than bundling multiple changes together. Aim for fewer than 500 lines changed when possible. Split large changes into multiple smaller PRs that can be reviewed independently. Smaller PRs are easier and faster to review, leading to quicker merge times and higher quality feedback.

### Commit Organization

Organize commits logically with each commit representing a complete, working change. Good commit organization might include separate commits for adding a feature, adding tests for that feature, and updating documentation. Avoid mixing unrelated changes in a single commit. Well-organized commits make it easier to understand the progression of changes and to bisect issues if problems arise later.

### Self-Review First

Before requesting review from others, review your own changes thoroughly. Check for any debug code or temporary changes that should not be committed. Verify there are no accidental changes to unrelated files. Ensure formatting is consistent with the codebase style. Test edge cases that the CI might not catch. This self-review catches obvious issues before they consume reviewer time.


## Code Review Process

### Requesting Review

When your PR is ready for review, mark it as ready if it was previously a draft. Request specific reviewers if you know who has relevant expertise. Add appropriate labels to categorize the PR. Link related issues in the description. Add comments on particularly complex areas of code to help reviewers understand your approach.

### Responding to Feedback

Engage constructively with review feedback. Acknowledge feedback and note when you have addressed it with a commit reference. Explain your decisions when you chose a particular approach for good reasons. Ask for clarification when feedback is unclear. If you disagree with feedback, express your perspective respectfully and be open to discussion.

### Making Changes

Address review comments promptly to keep the review process moving. Commit changes that address feedback with clear commit messages. You can amend commits if the changes are small corrections. Use force push with lease to update your PR branch safely while preserving the force push safety check.


## Review Guidelines

### For Reviewers

When reviewing PRs, examine code correctness to ensure the implementation is sound. Check test coverage to verify new code is properly tested. Verify documentation is updated to reflect changes. Consider performance implications of the changes. Evaluate security considerations especially for code handling user input or authentication. Ensure code style consistency with the rest of the codebase.

Provide constructive feedback with specific suggestions. Include code examples when they would clarify your point. Recognize good work when you see it. Remember that the goal is to improve the code while supporting the contributor.

### Review Comments

Good review feedback is specific and actionable. Instead of saying "This is wrong," explain what the issue is and suggest a solution. For example, you might suggest using a more idiomatic Rust pattern and show what the improved code would look like. This approach helps contributors learn and makes it clear how to address the feedback.


## CI/CD Checks

### Required Checks

All PRs must pass the automated CI checks before merging. These include cargo build for compilation verification, cargo test for unit tests, cargo fmt check for code formatting, cargo clippy for linting, and documentation builds. The CI runs automatically when you push changes to your PR branch.

### Fixing Failed Checks

When CI checks fail, fix the issues locally before pushing updates. Run cargo fmt to fix formatting issues. Run cargo clippy with the fix flag to automatically fix many linting issues. Run cargo test locally to debug test failures with the nocapture flag to see output. Fix all issues and push updates to trigger a new CI run.


## Merge Process

### Merge Requirements

Before a PR can be merged, all CI checks must pass, at least one maintainer must approve the changes, all review conversations must be resolved, the branch must be up-to-date with main, and there must be no merge conflicts.

### Merge Methods

Squash and merge is the preferred method for most PRs. This combines all commits into a single commit on main, keeping the history clean and making it easy to revert changes if needed. Rebase and merge preserves the individual commit history and is appropriate for PRs with well-organized, meaningful commits. Merge commits are rarely used and reserved for special circumstances.


## After Merge

### Clean Up

After your PR is merged, delete your local feature branch. GitHub automatically deletes the remote branch if configured to do so. Update your local main branch by checking out main and pulling the latest changes. This keeps your local repository clean and up-to-date.

### Follow Up

Monitor the codebase after your changes merge to catch any issues that emerge. Respond to questions from other contributors about your changes. Update related documentation if you discover gaps. Close any related issues that were not automatically closed by the PR.


## Common Issues

### Merge Conflicts

When merge conflicts occur, update your branch with the latest main by fetching and rebasing. Git will pause at each conflict, allowing you to resolve it. Edit the conflicted files to resolve the conflicts, add the resolved files, and continue the rebase. If the conflicts become too complex, you can abort the rebase and try a different approach.

### Large PR

If a PR becomes too large during development, consider closing it and splitting the work into smaller PRs. Create a tracking issue to coordinate the smaller PRs. Link all related PRs together so reviewers understand the bigger picture. Smaller, focused PRs are more likely to receive thorough review and merge quickly.

### Stale PR

If a PR goes without activity for an extended period, ping the reviewers with a comment. Rebase on the latest main to ensure the changes still apply cleanly. Add a comment explaining the current status. If the PR is no longer relevant, close it with an explanation so others know not to wait for it.


## Tips for Success

Communicate early about what you plan to implement to avoid wasted effort and get valuable design feedback. Test thoroughly rather than relying solely on CI since you understand your changes better than automated tests can. Be patient because reviewers have limited time and thorough review takes effort. Be responsive to feedback to keep the review process moving efficiently. Learn from reviews by treating feedback as an opportunity to improve your skills. Help others by reviewing other PRs when you have time, which builds goodwill and helps you learn the codebase.


## Summary

Successful pull requests are well-prepared with a clear purpose, properly documented and tested, responsive to feedback, and focused on a single change. Following these guidelines helps maintain code quality and makes the review process smooth for everyone involved. The time invested in creating a good PR pays off in faster reviews, fewer revision cycles, and a better end result.
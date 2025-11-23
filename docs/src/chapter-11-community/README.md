# Contributing to BotServer

Welcome to the BotServer community! We appreciate your interest in contributing to this open-source project. This chapter provides comprehensive guidance for contributors.

## Overview

BotServer is an open-source conversational AI platform built in Rust. We welcome contributions of all kinds:
- Code improvements
- Documentation updates
- Bug reports
- Feature suggestions
- Testing
- Community support

## Getting Started

### Prerequisites

Before contributing, ensure you have:
- Rust 1.70 or later
- PostgreSQL 14 or later
- Git experience
- Basic understanding of BotServer architecture

### First Steps

1. **Fork the Repository**
   ```bash
   git clone https://github.com/GeneralBots/BotServer.git
   cd BotServer
   ```

2. **Set Up Development Environment**
   - Follow the [Development Setup](./setup.md) guide
   - Run the bootstrap process
   - Verify everything works

3. **Find Something to Work On**
   - Check [good first issues](https://github.com/GeneralBots/BotServer/labels/good%20first%20issue)
   - Ask in discussions

## Types of Contributions

### Code Contributions

**Bug Fixes:**
- Reproduce the issue
- Write a test case
- Implement the fix
- Submit a pull request

**New Features:**
- Discuss in an issue first
- Design the solution
- Implement with tests
- Document the feature

**Performance Improvements:**
- Benchmark existing code
- Implement optimization
- Prove improvement with metrics
- Ensure no regressions

### Documentation

**Areas Needing Help:**
- API documentation
- BASIC keyword examples
- Tutorial content
- Translation

**Documentation Standards:**
- Clear and concise
- Include examples
- Keep up-to-date
- Follow markdown conventions

### Testing

**Test Coverage:**
- Unit tests for new code
- Integration tests for features
- Regression tests for bugs
- Performance benchmarks

**Testing Guidelines:**
- Test edge cases
- Mock external dependencies
- Keep tests fast
- Use descriptive names

## Development Process

### 1. Planning

Before starting work:
- Check existing issues
- Discuss major changes
- Get feedback on approach
- Claim the issue

### 2. Development

While coding:
- Follow [Code Standards](./standards.md)
- Write tests first (TDD)
- Keep commits atomic
- Update documentation

### 3. Testing

Before submitting:
- Run all tests locally
- Check code formatting
- Verify no warnings
- Test manually

### 4. Submission

Creating a pull request:
- Clear description
- Reference related issues
- Include test results
- Update CHANGELOG if needed

## Communication

### GitHub Discussions

For questions and ideas:
- Feature proposals
- Architecture discussions
- Community support
- General questions

### Issue Tracker

For specific problems:
- Bug reports
- Feature requests
- Documentation issues
- Performance problems

### Pull Requests

For code review:
- Implementation feedback
- Design discussions
- Testing verification
- Merge coordination

## Contribution Guidelines

### Code Quality

**Requirements:**
- Pass all tests
- No compiler warnings
- Formatted with `cargo fmt`
- Clean `cargo clippy` output

**Best Practices:**
- Write self-documenting code
- Add comments for complex logic
- Keep functions small
- Follow SOLID principles

### Commit Messages

**Format:**
```
type(scope): brief description

Detailed explanation if needed.

Fixes #issue-number
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Test additions
- `chore`: Maintenance

### Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature
   ```

2. **Make Changes**
   - Implement feature
   - Add tests
   - Update docs

3. **Submit PR**
   - Push to your fork
   - Open pull request
   - Fill template
   - Request review

4. **Code Review**
   - Address feedback
   - Update as needed
   - Maintain discussion

5. **Merge**
   - Squash if requested
   - Ensure CI passes
   - Maintainer merges

## Recognition

### Contributors

All contributors are recognized:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Community appreciation

### Core Team

Regular contributors may be invited to:
- Join core team
- Get merge permissions
- Mentor others

## Resources

### Documentation

- [Development Setup](./setup.md)
- [Code Standards](./standards.md)
- [Testing Guide](./testing.md)
- [Pull Request Guide](./pull-requests.md)

### Tools

- Rust toolchain
- VS Code with rust-analyzer
- Git and GitHub CLI
- LXC (optional)

### Learning

- Rust documentation
- BotServer architecture docs
- BASIC language reference
- Zitadel documentation

## Code of Conduct

We maintain a welcoming community:
- Be respectful
- Be inclusive
- Be collaborative
- Be professional

See our full [Code of Conduct](./code-of-conduct.md).

## Getting Help

### For Contributors

- Ask in pull requests
- Create discussion topics
- Review existing code
- Read documentation

### For Maintainers

- Provide clear feedback
- Be responsive
- Guide new contributors
- Maintain standards

## Legal

### Licensing

- BotServer uses AGPL-3.0 license
- Contributions must be compatible
- You retain copyright
- Grant project license rights

### Contributor Agreement

By contributing, you agree that:
- You have the right to contribute
- Your contribution is original
- You grant necessary licenses
- You follow the Code of Conduct

## Thank You!

Your contributions make BotServer better for everyone. Whether you're fixing a typo, adding a feature, or helping others, every contribution matters.

Welcome to the community!

## See Also

- [Development Setup](./development-setup.md) - Setting up your development environment
- [Testing](./testing.md) - Writing and running tests
- [Documentation](./documentation.md) - Contributing to documentation
- [IDE Extensions](./ide-extensions.md) - IDE integration and tooling
- [Chapter 1: Getting Started](../chapter-01/README.md) - Understanding the basics
- [Chapter 2: Packages](../chapter-02/README.md) - Bot package system
- [Chapter 5: BASIC Reference](../chapter-05/README.md) - Complete command reference
- [Chapter 6: Extensions](../chapter-06/README.md) - Extending BotServer
- [Chapter 9: Advanced Topics](../chapter-09/README.md) - Advanced features
- [Chapter 11: Infrastructure](../chapter-11/README.md) - Deployment and infrastructure
- [Chapter 12: Web API](../chapter-12/README.md) - REST and WebSocket APIs
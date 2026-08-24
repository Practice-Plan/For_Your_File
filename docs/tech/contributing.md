# Contributing Guidelines

Thank you for your interest in contributing to LNK File Management Center! This document provides guidelines and standards for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Code Style Guide](#code-style-guide)
5. [Commit Message Format](#commit-message-format)
6. [Pull Request Process](#pull-request-process)
7. [Testing Requirements](#testing-requirements)
8. [Documentation Standards](#documentation-standards)
9. [Issue Guidelines](#issue-guidelines)
10. [Community](#community)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inspiring community for all.

### Standards

- Be respectful and inclusive
- Welcome different perspectives
- Accept constructive criticism gracefully
- Focus on what's best for the community
- Show empathy towards others

### Enforcement

Violations may be reported to the project maintainers. All complaints will be reviewed and investigated.

## Getting Started

### Prerequisites

Before contributing, ensure you have:

1. **Read the documentation**
   - [Architecture](./architecture.md)
   - [API Reference](./api.md)
   - [Database Schema](./database-schema.md)

2. **Set up development environment**
   - Install Rust 1.77.2+
   - Install Node.js 18.x+
   - Install Windows SDK
   - See [Build & Deploy](./build-deploy.md)

3. **Fork and clone the repository**
   ```powershell
   git clone https://github.com/YOUR_USERNAME/lnk-file-management-center.git
   cd lnk-file-management-center
   git remote add upstream https://github.com/original/lnk-file-management-center.git
   ```

### First Contribution

Look for issues labeled:
- `good first issue` - Beginner-friendly
- `help wanted` - Contributions welcome
- `documentation` - Documentation improvements

## Development Workflow

### 1. Create a Branch

```powershell
# Update main branch
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-number
```

**Branch Naming Convention**:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Adding tests
- `chore/` - Maintenance tasks

### 2. Make Changes

- Write clean, readable code
- Follow code style guidelines
- Add tests for new functionality
- Update documentation
- Keep changes focused

### 3. Test Your Changes

```powershell
# Run Rust tests
cargo test

# Run frontend tests
npm test

# Run linting
cargo clippy
npm run lint

# Type checking
cargo check
npm run type-check
```

### 4. Commit Changes

Follow [Commit Message Format](#commit-message-format) guidelines.

### 5. Push and Create PR

```powershell
# Push your branch
git push origin feature/your-feature-name

# Create pull request via GitHub
```

## Code Style Guide

### Rust

Follow official Rust style guidelines: https://rust-lang.github.io/api-guidelines/

#### Naming Conventions

```rust
// Functions: snake_case
fn get_entry_by_id(id: i64) -> Result<Entry> { }

// Structs and Enums: PascalCase
pub struct EntryManager { }
pub enum ExpirationStatus { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_RETRIES: i32 = 3;

// Modules: snake_case
mod expiration_manager;
```

#### Code Organization

```rust
// File structure
// 1. Imports
use std::collections::HashMap;
use crate::models::Entry;

// 2. Constants and statics
const DEFAULT_TIMEOUT: u64 = 30;

// 3. Structs and enums
pub struct Manager { }

// 4. Implementations
impl Manager {
    // Public methods first
    pub fn new() -> Self { }

    // Private methods last
    fn internal_helper(&self) { }
}

// 5. Tests (in same file or tests/ module)
#[cfg(test)]
mod tests { }
```

#### Best Practices

```rust
// Use Result for error handling
pub fn create_entry(path: &str) -> Result<Entry> {
    // ...
}

// Document public APIs
/// Creates a new entry from the given path.
///
/// # Arguments
/// * `path` - Path to the LNK file
///
/// # Returns
/// The created entry
///
/// # Errors
/// Returns error if the file doesn't exist
pub fn create_entry(path: &str) -> Result<Entry> {
    // ...
}

// Use ? operator for error propagation
fn process_file(path: &str) -> Result<()> {
    let content = read_file(path)?;
    let parsed = parse_content(&content)?;
    save_entry(parsed)?;
    Ok(())
}

// Prefer iterators over loops
let entries: Vec<Entry> = paths
    .iter()
    .filter_map(|p| create_entry(p).ok())
    .collect();
```

#### Formatting

```powershell
# Format Rust code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy for linting
cargo clippy -- -D warnings
```

### TypeScript/React

Follow TypeScript and React best practices.

#### Naming Conventions

```typescript
// Interfaces: PascalCase
interface Entry { }

// Types: PascalCase
type ExpirationStatus = 'expired' | 'expiring_soon' | 'active';

// Functions: camelCase
function getEntryById(id: number): Entry { }

// Components: PascalCase
function EntryCard({ entry }: EntryCardProps) { }

// Constants: SCREAMING_SNAKE_CASE
const MAX_ENTRIES = 100;
```

#### Component Structure

```typescript
// Component file structure
import React from 'react';

// Types and interfaces
interface EntryCardProps {
  entry: Entry;
  onSelect: (id: number) => void;
}

// Component
export function EntryCard({ entry, onSelect }: EntryCardProps) {
  // Hooks at the top
  const [isExpanded, setIsExpanded] = useState(false);

  // Event handlers
  const handleClick = () => {
    onSelect(entry.id);
  };

  // Render
  return (
    <div className="entry-card">
      {/* ... */}
    </div>
  );
}
```

#### Best Practices

```typescript
// Use interfaces for object shapes
interface Entry {
  id: number;
  path: string;
}

// Use async/await for async operations
async function loadEntries(): Promise<Entry[]> {
  const response = await fetch('/api/entries');
  return response.json();
}

// Use optional chaining
const name = entry?.metadata?.name;

// Use nullish coalescing
const timeout = config.timeout ?? DEFAULT_TIMEOUT;

// Destructure props
function EntryCard({ entry, onSelect }: EntryCardProps) {
  // ...
}
```

#### Formatting

```powershell
# Format TypeScript code
npm run format

# Run ESLint
npm run lint

# Type checking
npm run type-check
```

### CSS/Tailwind

```typescript
// Use Tailwind utility classes
<div className="flex items-center gap-2 p-4 bg-white dark:bg-gray-800">

// For complex styles, use @apply or extract component
.card {
  @apply rounded-lg shadow-md p-4;
}
```

## Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

| Type | Description | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(hotkey): add global hotkey support` |
| `fix` | Bug fix | `fix(sync): resolve conflict detection issue` |
| `docs` | Documentation | `docs(api): update command reference` |
| `style` | Code style | `style: format code with cargo fmt` |
| `refactor` | Code refactoring | `refactor(db): improve query performance` |
| `test` | Adding tests | `test(expiration): add unit tests for manager` |
| `chore` | Maintenance | `chore: update dependencies` |
| `perf` | Performance | `perf(search): optimize FTS queries` |

### Scopes

Common scopes:
- `api` - Backend API commands
- `ui` - User interface
- `db` - Database operations
- `sync` - Cloud sync
- `hotkey` - Hotkey manager
- `expiration` - Expiration system
- `groups` - Group management
- `docs` - Documentation
- `build` - Build system

### Examples

#### Feature

```
feat(hotkey): add hotkey conflict detection

Implement conflict detection before registering a hotkey.
This prevents hotkeys from interfering with system shortcuts.

Closes #123
```

#### Bug Fix

```
fix(db): resolve transaction deadlock in batch operations

Use separate transactions for batch insert operations to avoid
database locks. Add retry logic for transient failures.

Fixes #456
```

#### Breaking Change

```
feat(api)!: change entry API response format

BREAKING CHANGE: The Entry struct now uses i64 for timestamps
instead of String. Update frontend code accordingly.

Migration guide:
- Change timestamp types from string to number
- Update date parsing logic
```

## Pull Request Process

### Before Creating PR

1. **Update your branch**
   ```powershell
   git checkout main
   git pull upstream main
   git checkout feature/your-feature
   git rebase main
   ```

2. **Run all checks**
   ```powershell
   cargo test
   cargo clippy
   cargo fmt --check
   npm test
   npm run lint
   npm run type-check
   ```

3. **Update documentation**
   - Update API docs if adding/modifying commands
   - Update architecture docs if changing system design
   - Update README if changing user-facing features

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Added unit tests
- [ ] Added integration tests
- [ ] Tested manually

## Checklist
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] No new warnings
- [ ] Tests pass locally
- [ ] PR title follows conventional commit format

## Related Issues
Closes #123
```

### Review Process

1. **Automated checks** run automatically
2. **Code review** by maintainers
3. **Testing** on multiple Windows versions
4. **Approval** from at least one maintainer
5. **Merge** by maintainer

### After Merge

- Delete your feature branch
- Update local main branch
- Start new feature branch for next work

## Testing Requirements

### Unit Tests

**Required for**:
- All public functions
- Complex logic
- Error handling paths
- Edge cases

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiration_status() {
        let entry = create_test_entry();
        let status = get_expiration_status(&entry);
        assert!(matches!(status, ExpirationStatus::NotExpiring));
    }

    #[test]
    fn test_expired_entry() {
        let mut entry = create_test_entry();
        entry.expires_at = Some(Utc::now().timestamp() - 1000);
        let status = get_expiration_status(&entry);
        assert!(matches!(status, ExpirationStatus::Expired { .. }));
    }
}
```

### Integration Tests

**Required for**:
- API endpoints
- Database operations
- File system operations
- IPC communication

**Example**:
```rust
#[cfg(test)]
mod integration_tests {
    use tauri::test::MockRuntime;

    #[test]
    fn test_add_entry_command() {
        let app = test_app();
        let result = invoke::create_entry(app, "C:\\test.lnk".to_string());
        assert!(result.is_ok());
    }
}
```

### Test Coverage

**Minimum coverage**: 80% for core modules

**Run coverage**:
```powershell
cargo tarpaulin --out Html
```

## Documentation Standards

### Code Documentation

**Required for**:
- All public functions
- All public structs and enums
- Complex private functions
- Modules

**Example**:
```rust
/// Expiration manager for tracking entry expiration dates.
///
/// The manager periodically checks for expired entries and notifies
/// the user through desktop notifications.
///
/// # Example
///
/// ```
/// let manager = ExpirationManager::new(conn);
/// let expired = manager.check_expired_entries()?;
/// ```
pub struct ExpirationManager {
    // ...
}
```

### API Documentation

Document in `docs/tech/api.md`:
- Function signature
- Parameters
- Return type
- Errors
- Examples

### Architecture Documentation

Update `docs/tech/architecture.md` when:
- Adding new components
- Changing system design
- Modifying data flow
- Adding new dependencies

### README Updates

Update README when:
- Adding user-facing features
- Changing installation process
- Updating requirements
- Modifying usage examples

## Issue Guidelines

### Bug Reports

Use the bug report template:

```markdown
**Description**
Clear description of the bug

**To Reproduce**
Steps to reproduce:
1. Go to '...'
2. Click on '...'
3. See error

**Expected Behavior**
What you expected to happen

**Actual Behavior**
What actually happened

**Environment**
- OS: Windows 11
- App Version: 0.0.3
- Rust Version: 1.77.2

**Screenshots**
If applicable

**Logs**
```
Paste relevant logs here
```

**Additional Context**
Any other context
```

### Feature Requests

Use the feature request template:

```markdown
**Is your feature request related to a problem?**
Clear description of the problem

**Describe the solution you'd like**
Clear description of what you want

**Describe alternatives you've considered**
Any alternative solutions

**Additional Context**
Any other context or screenshots

**Would you be willing to help implement this?**
[ ] Yes, I'd like to help implement this feature
```

### Issue Labels

- `bug` - Something isn't working
- `enhancement` - New feature or request
- `documentation` - Improvements to documentation
- `good first issue` - Good for newcomers
- `help wanted` - Extra attention needed
- `blocked` - Blocked by other issues
- `wontfix` - Will not be fixed

## Development Tips

### Debugging

**Rust**:
```powershell
# Enable debug logs
set RUST_LOG=debug

# Run with debugger
cargo run
```

**Frontend**:
```powershell
# Open DevTools (F12)
# Use React Developer Tools
# Check console for errors
```

### Performance Profiling

**Rust**:
```powershell
# Profile with cargo flamegraph
cargo flamegraph --root

# Benchmark tests
cargo bench
```

**Frontend**:
```powershell
# Use React DevTools Profiler
# Check bundle size
npm run analyze
```

### Database Debugging

```powershell
# Connect to database
sqlite3 %APPDATA%\wang.station\app\For_Your_File\lnk_management.db

# Check schema
.schema

# Check data
SELECT * FROM entries LIMIT 10;

# Check indexes
.indexes
```

## Community

### Getting Help

- **GitHub Issues**: For bugs and feature requests
- **Discussions**: For questions and general discussion
- **Wiki**: For community-contributed guides

### Contributing to Docs

Documentation improvements are always welcome:
- Fix typos and grammar
- Improve clarity
- Add examples
- Update screenshots

### Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- GitHub contributors page

## Questions?

If you have questions about contributing:
1. Check existing documentation
2. Search existing issues
3. Open a new discussion
4. Ask in the issue comments

Thank you for contributing to LNK File Management Center!
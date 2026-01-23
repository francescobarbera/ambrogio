# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Philosophy

Code like Kent Beck. Prioritize simplicity, readability, and testability. Make the code clear enough that it doesn't need comments to explain what it does.

### Working Style

- **Proceed step by step.** Keep code changes small and incremental so each implementation detail can be reviewed before moving on.
- **Understand before changing.** Read existing code before modifying it. Understand the context, patterns, and constraints already in place.
- **Minimal changes.** Make the smallest change that solves the problem. Avoid refactoring unrelated code, adding unnecessary abstractions, or "improving" things that weren't asked for.

## Documentation Maintenance

**Always update SPECIFICATIONS.md** when making changes to the codebase:
- Read `docs/SPECIFICATIONS.md` before starting work to understand the current state
- Update `docs/SPECIFICATIONS.md` after implementing new features or modifying existing behavior
- Keep the specifications in sync with the actual implementation

## Testing Requirements

**Always write tests.** When implementing a new feature or fixing a bug:
- Add unit tests for new functions and logic
- Add integration/e2e tests for new user-facing behavior
- Update existing tests when changing behavior
- Run the test suite after every change
- Ensure all tests pass before considering work complete

## Code Style

### General Principles

- Functions do one thing
- Names reveal intent
- Avoid primitive obsession: use domain types (e.g., `UserId` not `String`, `Money` not `float`)
- Make invalid states unrepresentable through the type system
- Prefer composition over inheritance

### Error Handling

- Use the language's idiomatic error handling (Result types, exceptions, etc.)
- Define specific error types per layer/module
- Propagate errors appropriately; don't swallow them silently
- Convert between error types at layer boundaries

### SOLID Principles

**S - Single Responsibility**: Each module/class has one reason to change.

**O - Open/Closed**: Open for extension, closed for modification. Use interfaces/traits.

**L - Liskov Substitution**: Implementations must be substitutable for their abstractions.

**I - Interface Segregation**: Prefer small, focused interfaces over large ones.

**D - Dependency Inversion**: High-level modules depend on abstractions, not concrete implementations.

## What NOT to Do

- Don't refactor unrelated code
- Don't add unnecessary comments, docstrings, or type annotations to unchanged code
- Don't add error handling for scenarios that can't happen
- Don't create abstractions for one-time operations
- Don't design for hypothetical future requirements

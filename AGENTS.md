# Agent Instructions

## Product

A space-trading economic simulation about merchants pursuing personal singularity through hypercapitalism.

## Stack

- Rust; edition and dependencies are defined in Cargo.toml.
- Bevy; use the version specified in Cargo.toml.
- Follow the APIs and conventions of the project's dependency versions.

## Coding

- Do not use `else` or `else if` branches. This is a deliberate project
  style rule. `let ... else` is allowed.
- Prefer guard clauses, early returns, `let ... else`, pattern matching,
  and focused helper functions.
- Keep changes focused on the requested task.

## Architecture

- Organize code and tests by domain, never by issue number.
- Put tests in separate files.
- Each new module gets its own directory.
- `mod.rs` contains only module declarations and re-exports.
  Types and logic belong in implementation files.
- Follow these rules for new code and code changed within the task.
  Do not reorganize unrelated existing code solely to enforce them.
- Larger architectural migrations require their own task.

## Content and Configuration

- Define concrete game content and balancing values in YAML.
  Define schemas, behaviour, and runtime state in Rust.
- Apply this separation to ships, enemies, weapons, abilities,
  upgrades, encounters, and similar configurable content.
- Validate content when loading: required fields, value ranges,
  unique identifiers, and references to other content.
- Report loading errors with the source file and affected field
  or identifier. Do not silently replace invalid content with defaults.
- Keep schema documentation and affected content consistent
  when changing a format.

## Bevy / ECS

- Model game objects as entities composed of components,
  not inheritance hierarchies.
- Store entity-specific state in components; use resources
  for shared game state and configuration.
- Implement gameplay behaviour in focused systems.
  Keep reusable calculations in ordinary Rust functions.
- Query only the components needed; request mutable access
  only when modifying data.
- Declare system ordering explicitly whenever correctness
  depends on execution order.
- Keep one authoritative representation of runtime state.
  Do not maintain duplicate state outside the ECS.

## Testing and Verification

- Test observable behaviour and acceptance criteria,
  not implementation details.
- Use unit tests for isolated rules and ECS integration tests
  for system interactions.
- Integration tests must exercise production systems.
  Tests of normal gameplay must also use production content.
- Use fixtures for invalid content and other cases that cannot
  reasonably be represented by production content.
- Control random seeds, simulation time, and inputs so automated
  tests are reproducible.
- For bug fixes, add a regression test that fails before the fix
  and passes afterwards where feasible.
- For Rust changes, run:
  - `cargo fmt --check`
  - `cargo check --locked`
  - Relevant tests with `cargo test --locked <filter>`
- Run the full `cargo test --locked` suite for changes affecting
  shared runtime behaviour, scheduling, content loading,
  or multiple domains.
- For content changes, run the relevant loading and validation tests.
- Report what passed, failed, or was not tested, including the reason
  for any omitted check.
- Do not claim a playtest unless the running game was actually
  operated and observed.
- For changes visible in the game, include short manual test steps
  and expected results in the pull request.

## GitHub Workflow

Every pull request body must include these two checklist items with this exact wording:

- [ ] i have done a self review
- [ ] i have done a playtest cycle

Include both items in every pull request, including documentation-only changes. Only the user may mark either item as checked; Codex must leave both unchecked and must not change their checkbox state.

- For issue-based development, read the issue and its acceptance
  criteria before making changes.
- Fetch origin before starting a new issue.
- Create the issue branch and worktree from the updated `origin/main`.
- Use branch names in the form `issue-<number>-<short-description>`.
- Reuse the existing branch, worktree, and pull request when continuing
  work on the same issue.
- Preserve unrelated local changes. Do not overwrite or discard them.
- For implementation tasks without an issue, follow the user's
  requested scope; creating an issue is not a prerequisite.
- After completing implementation and verification, open a pull
  request unless the user requested local-only work.
- Explain the problem, the resulting behaviour, and how the change
  was verified. Link the issue when one exists.
- Reviews, analysis, and advice do not require a branch, worktree,
  or pull request unless changes are requested.

## Definition of Done

- The requested task and applicable acceptance criteria are fulfilled.
- Relevant checks have passed, or remaining failures and limitations
  are explicitly reported.
- Changed content and documentation are consistent with the code.
- Changes visible in the game can be verified by starting the game
  and following the documented manual test steps.
- The pull request is ready for review when one is required.

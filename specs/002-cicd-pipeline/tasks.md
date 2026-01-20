# Tasks: CI/CD Pipeline

**Input**: Design documents from `/specs/002-cicd-pipeline/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are NOT explicitly requested in the feature specification. Test tasks are omitted.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Based on plan.md structure (GitHub Actions workflows):
- **Workflows**: `.github/workflows/`
- **CI Workflow**: `.github/workflows/ci.yml`
- **Release Workflow**: `.github/workflows/release.yml`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create directory structure for GitHub Actions workflows

- [x] T001 Create `.github/workflows/` directory structure
- [x] T002 [P] Add `.gitignore` entries for any CI-related temporary files if needed

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core workflow structure that MUST be complete before specific jobs can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Create initial `.github/workflows/ci.yml` with workflow name and trigger configuration (push to all branches, pull_request events)
- [x] T004 Add concurrency configuration to ci.yml (group by ref, cancel-in-progress: true) per FR-009
- [x] T005 [P] Create initial `.github/workflows/release.yml` with workflow name and tag trigger (v*.*.* pattern) per FR-010
- [x] T006 Add permissions block to release.yml (contents: write) for release creation

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Automated Code Quality Checks (Priority: P1) MVP

**Goal**: Automatically validate code changes with formatting and linting checks on every push

**Independent Test**: Push code with formatting issues to any branch, verify lint job runs and reports violations

### Implementation for User Story 1

- [x] T007 [US1] Add `lint` job to ci.yml with ubuntu-latest runner
- [x] T008 [US1] Add checkout step using actions/checkout@v4 in lint job
- [x] T009 [US1] Add Rust toolchain setup using dtolnay/rust-toolchain@stable with rustfmt, clippy components in lint job
- [x] T010 [US1] Add dependency caching using Swatinem/rust-cache@v2 in lint job per FR-014
- [x] T011 [US1] Add `cargo fmt --all -- --check` step for formatting verification per FR-003
- [x] T012 [US1] Add `cargo clippy --all-targets --all-features -- -D warnings` step for linting per FR-004

**Checkpoint**: User Story 1 complete - code quality checks run on every push

---

## Phase 4: User Story 2 - Automated Test Execution (Priority: P1)

**Goal**: Run all tests automatically when code is pushed

**Independent Test**: Push code with a failing test, verify test job runs and reports the failure

### Implementation for User Story 2

- [x] T013 [US2] Add `test` job to ci.yml with ubuntu-latest runner
- [x] T014 [US2] Add checkout step using actions/checkout@v4 in test job
- [x] T015 [US2] Add Rust toolchain setup using dtolnay/rust-toolchain@stable in test job
- [x] T016 [US2] Add dependency caching using Swatinem/rust-cache@v2 in test job per FR-014
- [x] T017 [US2] Add `cargo test --all-features` step for test execution per FR-005

**Checkpoint**: User Story 2 complete - tests run automatically on every push

---

## Phase 5: User Story 3 - Build Verification (Priority: P1)

**Goal**: Build the project automatically and make artifacts downloadable

**Independent Test**: Push code, verify build job produces downloadable artifact

### Implementation for User Story 3

- [x] T018 [US3] Add `build` job to ci.yml with ubuntu-latest runner
- [x] T019 [US3] Add checkout step using actions/checkout@v4 in build job
- [x] T020 [US3] Add Rust toolchain setup using dtolnay/rust-toolchain@stable in build job
- [x] T021 [US3] Add dependency caching using Swatinem/rust-cache@v2 in build job per FR-014
- [x] T022 [US3] Add `cargo build --release` step for release build per FR-006
- [x] T023 [US3] Add artifact upload using actions/upload-artifact@v4 for `target/release/speck` with 7-day retention per FR-007

**Checkpoint**: User Story 3 complete - builds run and artifacts are downloadable

---

## Phase 6: User Story 4 - Pull Request Gating (Priority: P2)

**Goal**: Ensure PRs show check status and can be blocked until checks pass

**Independent Test**: Create PR with failing checks, verify GitHub shows failed status on PR

### Implementation for User Story 4

- [x] T024 [US4] Verify ci.yml triggers on pull_request events (already added in T003) per FR-002
- [x] T025 [US4] Document branch protection setup instructions in quickstart.md (require lint, test, build checks) per FR-008
- [x] T026 [US4] Add status check names to workflow jobs for clear GitHub UI display

**Checkpoint**: User Story 4 complete - PR checks visible and can be required for merge

---

## Phase 7: User Story 5 - Release Artifact Creation (Priority: P3)

**Goal**: Automatically create releases with binaries when version tags are pushed

**Independent Test**: Push a v0.1.0 tag, verify release is created with binary attached

### Implementation for User Story 5

- [x] T027 [US5] Add `release` job to release.yml with ubuntu-latest runner
- [x] T028 [US5] Add checkout step using actions/checkout@v4 in release job
- [x] T029 [US5] Add Rust toolchain setup using dtolnay/rust-toolchain@stable in release job
- [x] T030 [US5] Add dependency caching using Swatinem/rust-cache@v2 in release job per FR-014
- [x] T031 [US5] Add `cargo build --release` step in release job
- [x] T032 [US5] Add packaging step to create `speck-${{ github.ref_name }}-linux-x86_64.tar.gz` archive per FR-011
- [x] T033 [US5] Add release creation using softprops/action-gh-release@v1 with generate_release_notes: true per FR-011

**Checkpoint**: User Story 5 complete - releases created automatically on version tags

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T034 [P] Validate all workflow YAML syntax with `actionlint` or online validator
- [ ] T035 [P] Test complete CI workflow by pushing a test commit
- [ ] T036 [P] Test release workflow by creating a test tag (v0.0.1-test)
- [ ] T037 Review and clean up any test tags/releases created during validation
- [ ] T038 Run quickstart.md validation (follow setup steps in clean environment)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1, US2, US3 can proceed in parallel (different jobs in same file, but order matters for readability)
  - US4 depends on US1-3 being complete (needs checks to exist)
  - US5 is independent (separate workflow file)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: FOUNDATION - Can start after Foundational phase
- **User Story 2 (P1)**: Can start after Foundational (parallel with US1 if using separate edits)
- **User Story 3 (P1)**: Can start after Foundational (parallel with US1/2 if using separate edits)
- **User Story 4 (P2)**: Depends on US1, US2, US3 (needs all CI jobs to exist for branch protection)
- **User Story 5 (P3)**: Can start after Foundational (separate workflow file, independent)

### Within Each User Story

- Checkout step before toolchain setup
- Toolchain setup before cache
- Cache before build/test commands
- Build commands before artifact upload

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T003 and T005 can run in parallel (different workflow files)

**Phase 3-5 (US1, US2, US3)**:
- These modify the same ci.yml file, so tasks within each story are sequential
- However, US5 (release.yml) can be done in parallel with US1-3

**Phase 8 (Polish)**:
- T034, T035, T036 can run in parallel (different validation activities)

---

## Parallel Example: CI Jobs Setup

```bash
# After foundational phase, these can be developed in parallel on different branches:
# Branch A: User Story 1 (lint job)
Task: "Add lint job to ci.yml"

# Branch B: User Story 5 (release workflow)
Task: "Add release job to release.yml"
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - creates base workflow files)
3. Complete Phase 3: User Story 1 (lint checks)
4. Complete Phase 4: User Story 2 (test execution)
5. Complete Phase 5: User Story 3 (build + artifacts)
6. **STOP and VALIDATE**: Push code, verify all checks run
7. This delivers core CI value - every push is validated

### Full Feature (All Stories)

1. Complete MVP (US1 + US2 + US3)
2. Add User Story 4 (PR gating documentation)
3. Add User Story 5 (release automation)
4. Polish phase for production readiness

### Single Developer Strategy

1. Complete Setup + Foundational
2. Add all CI jobs sequentially (US1 → US2 → US3) in single ci.yml
3. Add PR documentation (US4)
4. Add release workflow (US5)
5. Validate and polish

---

## Notes

- [P] tasks = different files, no dependencies within same phase
- [US#] label maps task to specific user story for traceability
- Each user story should be independently completable after its dependencies
- Commit after each phase or logical group
- Stop at any checkpoint to validate story independently
- All jobs run in parallel by default (no `needs:` between lint/test/build)

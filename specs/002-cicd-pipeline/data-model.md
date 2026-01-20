# Data Model: CI/CD Pipeline

**Feature**: 002-cicd-pipeline
**Date**: 2026-01-20

## Overview

This document describes the structure of GitHub Actions workflow files. Unlike traditional data models with database entities, CI/CD pipelines are defined declaratively in YAML. The "entities" here are workflow components.

## Workflow Components

### Workflow

The top-level container for a CI/CD pipeline.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | Yes | Display name shown in GitHub UI |
| on | object | Yes | Trigger conditions (push, pull_request, etc.) |
| concurrency | object | No | Concurrency control settings |
| permissions | object | No | GITHUB_TOKEN permissions |
| env | object | No | Environment variables for all jobs |
| jobs | object | Yes | Map of job definitions |

### Trigger (on)

Defines when the workflow runs.

| Trigger Type | Use Case | Configuration |
|--------------|----------|---------------|
| push | Run on every push | `branches: ['**']` or specific branches |
| pull_request | Run on PR events | `types: [opened, synchronize, reopened]` |
| push.tags | Run on version tags | `tags: ['v*.*.*']` |

### Concurrency

Controls parallel execution of workflow runs.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| group | string | Yes | Grouping key (e.g., `${{ github.workflow }}-${{ github.ref }}`) |
| cancel-in-progress | boolean | No | Cancel running jobs when new run starts |

### Job

A unit of work within a workflow.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | Display name (defaults to job key) |
| runs-on | string | Yes | Runner environment (e.g., `ubuntu-latest`) |
| needs | string[] | No | Job dependencies (run after these complete) |
| steps | array | Yes | Ordered list of steps to execute |
| env | object | No | Job-specific environment variables |
| timeout-minutes | number | No | Maximum job duration |

### Step

A single command or action within a job.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| name | string | No | Display name for the step |
| uses | string | No* | Action to use (e.g., `actions/checkout@v4`) |
| run | string | No* | Shell command to execute |
| with | object | No | Input parameters for action |
| env | object | No | Step-specific environment variables |
| if | string | No | Conditional execution expression |

*Either `uses` or `run` is required, not both.

### Artifact

Files produced by a workflow for later use.

| Field | Type | Description |
|-------|------|-------------|
| name | string | Artifact identifier |
| path | string | Path(s) to upload |
| retention-days | number | How long to keep (default: 90) |

## Workflow Relationships

```
Workflow (1)
    │
    ├── Triggers (1..n) ─── push, pull_request, tags
    │
    ├── Concurrency (0..1)
    │
    └── Jobs (1..n)
            │
            ├── Dependencies (0..n) ─── other jobs via "needs"
            │
            └── Steps (1..n)
                    │
                    └── Artifacts (0..n) ─── via upload-artifact action
```

## State Transitions

### Workflow Run States

```
queued → in_progress → completed
                          │
                          ├── success
                          ├── failure
                          ├── cancelled
                          └── skipped
```

### Job States

```
queued → in_progress → completed
    │                      │
    └── waiting ──────────┘ (if has "needs" dependencies)
```

## CI Workflow Structure

```yaml
ci.yml:
  ├── name: "CI"
  ├── on: [push, pull_request]
  ├── concurrency: {group, cancel-in-progress}
  └── jobs:
      ├── lint:
      │   ├── runs-on: ubuntu-latest
      │   └── steps: [checkout, toolchain, cache, fmt, clippy]
      ├── test:
      │   ├── runs-on: ubuntu-latest
      │   └── steps: [checkout, toolchain, cache, test]
      └── build:
          ├── runs-on: ubuntu-latest
          └── steps: [checkout, toolchain, cache, build, upload-artifact]
```

## Release Workflow Structure

```yaml
release.yml:
  ├── name: "Release"
  ├── on: push.tags (v*.*.*)
  ├── permissions: {contents: write}
  └── jobs:
      └── release:
          ├── runs-on: ubuntu-latest
          └── steps: [checkout, toolchain, cache, build-release, package, create-release]
```

## Validation Rules

1. **Workflow name** must be unique within `.github/workflows/`
2. **Job keys** must be unique within a workflow
3. **Step names** should be unique within a job for clarity
4. **needs** references must point to existing job keys
5. **Concurrency group** should include `github.ref` to avoid cross-branch conflicts
6. **Tag pattern** must match semantic versioning for releases

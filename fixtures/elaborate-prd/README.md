# Elaborate PRD Fixture

Comprehensive test fixture demonstrating all PRD YAML features from SPEC-035.

## Purpose

This fixture showcases every field and status type available in the PRD format:

- **All status types**: pending, in_progress, done, blocked, skipped
- **All priority levels**: low, medium, high, critical
- **Dependencies**: tasks with `depends_on` references
- **Blocked tasks**: with `blocked_by` explanations
- **Acceptance criteria**: detailed completion requirements
- **Tags**: categorization and filtering
- **Timestamps**: created, updated, completed dates

## Usage

Launch Ralph with this fixture:

```bash
just dev-fixtures elaborate-prd
```

Or directly:

```bash
ralph --project /home/vince/Projects/ralph4days/fixtures/elaborate-prd
```

## Contents

**Project**: E-commerce platform with 15 tasks across backend, frontend, infrastructure, and testing.

### Task Breakdown

- **Done** (3): Database schema, REST API, Product catalog UI
- **In Progress** (2): Stripe payments, Shopping cart
- **Blocked** (2): CI/CD pipeline, Inventory management
- **Pending** (6): Auth UI, Order history, Search, Reviews, E2E tests, etc.
- **Skipped** (2): API docs, Social sharing

## Testing the PRD Viewer

This fixture is ideal for testing the PRD viewer UI because it demonstrates:

1. All task status badges and colors
2. Priority badges (low → critical)
3. Task dependencies visualization
4. Blocked task reasons
5. Acceptance criteria lists
6. Tag display
7. Timestamp formatting

## Reset

The fixture has no reset script because it's read-only. The PRD never changes during testing.

# Specification Format Standard

| Field | Value |
|-------|-------|
| **Spec ID** | SPEC-000 |
| **Title** | Specification Format Standard |
| **Status** | Active |
| **Version** | 1.0.0 |
| **Created** | 2026-02-05 |
| **Author** | Vince Liem |
| **Co-Author** | Claude Opus 4.5 |

---

## 1. Purpose

This specification defines the standard format for all specifications in the Ralph Loop project. It establishes:

- The structure and metadata for specification documents
- The numbering convention for specifications (living numbers)
- Traceability requirements linking specs to implementation

## 2. Scope

This specification applies to all documents in `.specs/` directories.

## 3. Definitions

| Term | Definition |
|------|------------|
| **Specification** | A normative document defining requirements that implementations MUST, SHOULD, or MAY follow |
| **Living Document** | A document that evolves with the project, reorganized as needed |
| **Requirement** | A single, testable statement of what the system must do |
| **Traceability** | The ability to link requirements to their implementation and tests |

## 4. Numbering Conventions

### 4.1 Living Numbers (`.specs/`)

Specifications use **three-digit living numbers** with gaps for insertion:

```
.specs/
├── 000_SPECIFICATION_FORMAT.md   # Meta: how to write specs
├── 010_TRACEABILITY.md           # Traceability standard
├── 020_ANTI_GAMING.md            # Anti-gaming rules
├── 060_TESTING_STANDARDS.md      # Testing standards
└── ...
```

**Rules:**

| Rule ID | Requirement |
|---------|-------------|
| NUM-01 | Numbers SHALL be three digits, assigned in increments of 10 (000, 010, 020...) |
| NUM-02 | When inserting between existing specs, use the midpoint (e.g., 015 between 010 and 020) |
| NUM-03 | If no midpoint exists, renumber the affected section |
| NUM-04 | Numbers represent **logical grouping**, not creation order |
| NUM-05 | Related specs SHOULD be numerically adjacent |

## 5. Specification Hierarchy

### 5.1 Spec Locations

```
ralph4days/
├── .specs/                       # Meta specs + cross-cutting standards
│   ├── 000_SPECIFICATION_FORMAT.md
│   ├── 010_TRACEABILITY.md
│   ├── 020_ANTI_GAMING.md
│   └── 060_TESTING_STANDARDS.md
│
├── src/
│   └── .specs/                   # Frontend specs (React components, UI)
│       └── ...
│
└── src-tauri/
    └── .specs/                   # Backend specs (Rust loop engine)
        └── ...
```

### 5.2 Spec ID Namespaces

| Prefix | Namespace | Location |
|--------|-----------|----------|
| `SPEC-0XX` | Meta specs | `/.specs/` |
| `SPEC-FXX` | Frontend specs | `/src/.specs/` |
| `SPEC-BXX` | Backend specs | `/src-tauri/.specs/` |

## 6. Specification Document Structure

### 6.1 Required Metadata

Every specification MUST begin with a metadata table:

```markdown
# [Title]

| Field | Value |
|-------|-------|
| **Spec ID** | SPEC-NNN |
| **Title** | [Human-readable title] |
| **Status** | Draft | Active | Deprecated |
| **Version** | [semver] |
| **Created** | [YYYY-MM-DD] |
| **Author** | [author] |
| **Supersedes** | [optional: SPEC-XXX] |
| **Traces To** | [optional: source files] |
```

### 6.2 Status Values

| Status | Meaning |
|--------|---------|
| **Draft** | Under development, not yet normative |
| **Active** | Current specification, implementations MUST comply |
| **Deprecated** | Superseded, kept for reference only |

### 6.3 Required Sections

| Section | Purpose |
|---------|---------|
| **1. Purpose** | Why this specification exists |
| **2. Scope** | What this specification covers and excludes |
| **3. Definitions** | Terms used in this specification |
| **4+. Requirements** | The actual normative content |

### 6.4 Requirement Format

Requirements MUST be:

1. **Uniquely identified** with a requirement ID
2. **Testable** — can be verified as met or not met
3. **Atomic** — one requirement per statement
4. **Traceable** — linked to implementation

**Format:**

```markdown
### REQ-NNN-MM: [Requirement Title]

[Requirement statement using RFC 2119 keywords]

| Traces To | `path/to/implementation.rs:function_name` |
| Tested By | `test_name` or "Manual verification" |
| Rationale | [Why this requirement exists] |
```

## 7. RFC 2119 Keywords

Specifications use RFC 2119 keywords with precise meanings:

| Keyword | Meaning |
|---------|---------|
| **MUST** | Absolute requirement. Implementations violating this are non-compliant. |
| **MUST NOT** | Absolute prohibition. |
| **SHOULD** | Recommended. Valid reasons may exist to ignore, but implications must be understood. |
| **MAY** | Optional. Implementations may or may not include this. |

## 8. File Naming

### 8.1 Format

```
NNN_UPPER_SNAKE_CASE.md
```

| Component | Rule |
|-----------|------|
| `NNN` | Three-digit living number (000, 010, 015, 020...) |
| `_` | Underscore separator |
| `NAME` | UPPER_SNAKE_CASE descriptive name |
| `.md` | Markdown extension |

## 9. Versioning

### 9.1 Semantic Versioning

Specifications follow semantic versioning:

| Change Type | Version Bump | Example |
|-------------|--------------|---------|
| Breaking requirement change | Major | 1.0.0 → 2.0.0 |
| New requirements (backward compatible) | Minor | 1.0.0 → 1.1.0 |
| Clarifications, typo fixes | Patch | 1.0.0 → 1.0.1 |

### 9.2 Change Log

Specifications SHOULD include a change log section:

```markdown
## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-05 | Initial specification |
| 1.1.0 | 2026-02-15 | Added REQ-XXX-YY |
```

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-05 | Initial specification adapted from iuppiter-dar |

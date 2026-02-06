# Dev Project — Bookmarks Manager

**Purpose**: Comprehensive mid-progress fixture exercising every frontend rendering path.

20 tasks across 5 features and 7 disciplines. Covers all status/priority combos,
dependency chains up to 3 deep, blocked_by reasons, 0–4 acceptance criteria,
and varied timestamps.

## Usage

```bash
cargo test --manifest-path src-tauri/Cargo.toml --test generate_fixtures generate_all_fixtures -- --nocapture --test-threads=1
just reset-mock
just dev-mock 04-dev-project
```

## What this exercises

- **TaskDetailSidebar**: all 5 status badges, all 4 priority badges, blocked_by alert,
  depends_on badges, acceptance criteria list, tags, created/updated/completed timestamps
- **PlaylistView**: blocked+skipped in "Issues", done section, in_progress NOW PLAYING, pending
- **FeaturesPage**: 5 features with varied completion %
- **DisciplinesPage**: 7 disciplines with tasks, 3 with 0 tasks
- **Filters**: 14 distinct tags, every status/priority combo, text search on titles+descriptions
- **TaskIdDisplay**: multiple feature+discipline acronym combos

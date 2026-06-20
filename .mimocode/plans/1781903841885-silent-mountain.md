# Plan: Commit Untracked and Modified Files

## Files to Commit (excluding package.json, Cargo.toml, tauri.conf.json)

### Modified:
- `angular.json`

### Untracked:
- `.mimocode/plans/1781902392021-witty-knight.md`
- `src/app/entities/response.model.ts`
- `src/app/entities/error.model.ts`
- `src/app/api/tauri-api.service.ts`
- `src-tauri/src/commands/crud.macro.rs`
- `src-tauri/src/utils/mod.rs`
- `src-tauri/src/utils/response.utils.rs`

## Proposed Commits (logical groups)

### Commit 1: `feat(flatpak): add utils and commands modules`
- `src-tauri/src/commands/crud.macro.rs`
- `src-tauri/src/utils/mod.rs`
- `src-tauri/src/utils/response.utils.rs`

### Commit 2: `feat(frontend): add api service and entity models`
- `src/app/api/tauri-api.service.ts`
- `src/app/entities/response.model.ts`
- `src/app/entities/error.model.ts`

### Commit 3: `chore: update angular config`
- `angular.json`

### Commit 4: `docs: add project plan` (SKIP - user excluded)

## Execution
1. Stage and commit each group in order
2. Verify `git status` is clean

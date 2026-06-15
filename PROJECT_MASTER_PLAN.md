# PROJECT MASTER PLAN: Translator

**Project:** Translator
**Document Version:** 2.0
**Date:** 2026-06-14
**Status:** Logging v2 Updated - Needs TauriApi Fix

---

## Part 1: Project Overview

### Purpose

Multi-language translation application.

### Tech Stack

- **Frontend:** Angular v19
- **Backend:** Rust (Tauri v2)

### Build Status

- TypeScript: ✅ PASS
- Rust: ✅ PASS
- File Logging: 🔄 Needs Tauri command
- TauriApi: ⚠️ STATIC (not injectable)

---

## Part 2: Logging System

### TypeScript

- ✅ Updated to `_shared/logging.service.ts` v2
- ✅ Good environment configuration

### Rust

- ✅ Updated to `_shared/rust/logging/` module
- ✅ Timestamp-based file naming
- 🔄 Needs Tauri command `write_log_to_file`

---

## Part 3: CRITICAL ISSUE: Static TauriApiService

### Current (WRONG)

```typescript
// api/tauri-api.service.ts
export class TauriApiService {
  static async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(cmd, args);
  }
}

// Usage - STATIC
const result = await TauriApiService.invoke('translate_text', {...});
```

### Problems

1. Not injectable - can't use DI
2. No error handling
3. No logging
4. No timeout
5. Direct `invoke()` import in translation.service.ts

### Required Fix

```typescript
// api/tauri-api.service.ts - CORRECT
@Injectable({ providedIn: "root" })
export class TauriApiService {
  constructor(private loggingService: LoggingService) {}

  async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    this.loggingService.debug(`Invoking: ${cmd}`);
    try {
      const result = await invoke<T>(cmd, args);
      return result;
    } catch (error) {
      this.loggingService.error(`Invoke failed: ${cmd}`, error);
      throw error;
    }
  }
}
```

---

## Part 4: Implementation Tasks

### Phase 1: CRITICAL - Fix TauriApiService

- [ ] Change from static to injectable class
- [ ] Add error handling with LoggingService
- [ ] Add timeout handling
- [ ] Remove direct `import { invoke }` from translation.service.ts

### Phase 2: File Logging Integration

- [ ] Add `write_log_to_file` Tauri command
- [ ] Register in `lib.rs`

---

## Part 5: Key Files

### Files to MODIFY

| File                                                   | Action                   | Priority |
| ------------------------------------------------------ | ------------------------ | -------- |
| `api/tauri-api.service.ts`                             | Make injectable          | 🔴 HIGH  |
| `features/translation/services/translation.service.ts` | Use injectable TauriApi  | 🔴 HIGH  |
| `src-tauri/src/lib.rs`                                 | Add file logging command | 🔴 HIGH  |

---

## Part 6: Verification

```bash
# Check for static invoke
grep -rn "static.*invoke\|TauriApiService.invoke" src/app --include="*.ts"
# Should return 0 results after fix

# Check for direct invoke import
grep -rn "import.*invoke.*from.*@tauri-apps" src/app --include="*.ts"
# Should return 0 results after fix

# TypeScript check
npx tsc --noEmit
```

---

**Document Status:** v2.0 - CRITICAL: Static TauriApiService
**Last Updated:** 2026-06-14

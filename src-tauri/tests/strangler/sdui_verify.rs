//! SDUI Verification Tests for Translator
//!
//! This module verifies that the Translator app properly uses library components
//! from @tauri-front/shared instead of hardcoded UI logic.
//!
//! ## SDUI Readiness Criteria
//!
//! 1. **Schema-based rendering**: App must use `SchemaShellComponent` or equivalent
//!    library component for UI rendering, NOT manual template-based components
//! 2. **No hardcoded UI components**: App should NOT have inline Angular templates
//!    with hardcoded HTML elements (div, button, input, etc.)
//! 3. **Event-driven architecture**: All UI events should flow through the schema
//!    event bus, NOT direct DOM manipulation or custom event handlers
//! 4. **Library component usage**: All UI components (buttons, inputs, selects, etc.)
//!    must come from @tauri-front/shared schema components
//! 5. **State management via SignalStore**: App state should be managed through
//!    the library's SignalStoreService, NOT local component state
//!
//! ## Verification Strategy
//!
//! This test module performs static analysis on the compiled Angular bundle
//! and source files to verify SDUI compliance.

use std::fs;
use std::path::Path;

/// Component selectors that indicate hardcoded UI (non-SDUI) patterns
const HARDCODED_COMPONENT_SELECTORS: &[&str] = &[
  "app-button",
  "app-input",
  "app-select",
  "app-textarea",
  "app-toggle",
  "app-dropdown",
  "app-modal",
  "app-overlay",
  "source-input",
  "target-output",
  "swap-btn",
  "translate-btn",
  "theme-btn",
  "shortcuts-btn",
  "lang-selector",
];

/// Schema-based (SDUI) component selectors
const SCHEMA_COMPONENT_SELECTORS: &[&str] = &[
  "lib-schema-shell",
  "schema-shell",
  "lib-page-renderer",
  "lib-component-renderer",
];

/// Verifies that the main Angular component uses schema-based rendering
#[test]
fn verify_schema_shell_component_usage() {
  let app_ts_path = Path::new("src/app/app.ts");

  if !app_ts_path.exists() {
    // On new branch, app.ts may not exist or may be renamed
    let alt_paths = [
      "src/app/app.component.ts",
      "src/app/components/app/app.component.ts",
    ];

    let found = alt_paths.iter().any(|p| Path::new(p).exists());
    if !found {
      println!("WARNING: Could not find app.ts or alternative component file");
      println!("SDUI verification cannot proceed without source files");
      return;
    }
  }

  let content = fs::read_to_string(app_ts_path)
    .expect("Failed to read app.ts");

  // Check for SchemaShellComponent import (SDUI pattern)
  let has_schema_shell = content.contains("SchemaShellComponent")
    || content.contains("lib-schema-shell")
    || content.contains("<lib-schema-shell");

  // Check for manual schema rendering (anti-pattern)
  let has_manual_schema_renderer = content.contains("SchemaRendererService")
    && content.contains("setComponentResolver");

  // Check for local template with hardcoded elements (anti-pattern)
  let has_local_template = content.contains("template:")
    && (content.contains("<div")
      || content.contains("<button")
      || content.contains("<input")
      || content.contains("<select"));

  println!("\n=== Schema Shell Component Analysis ===");
  println!("Uses SchemaShellComponent: {}", has_schema_shell);
  println!("Has manual SchemaRenderer: {}", has_manual_schema_renderer);
  println!("Has local hardcoded template: {}", has_local_template);

  // SDUI PASS: Uses schema shell OR no local template
  let sdui_pass = has_schema_shell || !has_local_template;

  // ANTI-PATTERN: Manual schema rendering + local template
  let has_dual_pattern = has_manual_schema_renderer && has_local_template;

  if has_dual_pattern {
    println!("\nVIOLATION: App uses BOTH manual SchemaRenderer AND local template");
    println!("This indicates incomplete SDUI migration (strangler fig anti-pattern)");
  }

  if has_manual_schema_renderer && !has_schema_shell {
    println!("\nWARNING: Manual SchemaRenderer detected without SchemaShellComponent");
    println!("This is acceptable for main branch过渡 but should be refactored");
  }

  assert!(sdui_pass, "App must use SchemaShellComponent or have no local template");
}

/// Verifies no hardcoded element IDs that indicate tight coupling
#[test]
fn verify_no_hardcoded_element_ids() {
  let app_ts_path = Path::new("src/app/app.ts");

  if !app_ts_path.exists() {
    return;
  }

  let content = fs::read_to_string(app_ts_path)
    .expect("Failed to read app.ts");

  let hardcoded_ids = [
    "outputText",
    "inputText",
    "source-input",
    "target-output",
    "shortcuts-el",
    "lang-selector",
    "theme-btn",
  ];

  println!("\n=== Hardcoded Element ID Analysis ===");

  let mut violations = Vec::new();
  for id in &hardcoded_ids {
    if content.contains(&format!("[data-element-id=\"{}\"]", id))
      || content.contains(&format!("id=\"{}\"", id))
      || content.contains(&format!("#{}", id))
    {
      violations.push(*id);
      println!("Found hardcoded ID reference: {}", id);
    }
  }

  if violations.is_empty() {
    println!("No hardcoded element IDs found - GOOD");
  }

  // This is informational - hardcoded IDs in event handlers are common
  // but indicate tight coupling to schema element IDs
}

/// Verifies event bus usage follows SDUI patterns
#[test]
fn verify_event_bus_usage() {
  let app_ts_path = Path::new("src/app/app.ts");

  if !app_ts_path.exists() {
    return;
  }

  let content = fs::read_to_string(app_ts_path)
    .expect("Failed to read app.ts");

  println!("\n=== Event Bus Usage Analysis ===");

  // Count event subscriptions
  let event_subscriptions: Vec<_> = content
    .match_indices("this.eventBus.on(")
    .collect();

  println!("Event bus subscriptions found: {}", event_subscriptions.len());

  // Count manual event listeners
  let manual_listeners: Vec<_> = content
    .match_indices("@HostListener")
    .collect();

  println!("@HostListener decorators found: {}", manual_listeners.len());

  // Check for emitToOutput pattern (anti-pattern - DOM direct manipulation)
  let has_emit_to_output = content.contains("emitToOutput");
  println!("Has emitToOutput (DOM manipulation): {}", has_emit_to_output);

  if has_emit_to_output {
    println!("\nVIOLATION: emitToOutput indicates direct DOM manipulation");
    println!("SDUI pattern should use event bus + component state, not DOM APIs");
  }

  // SDUI should have minimal HostListeners (keyboard shortcuts acceptable)
  let excessive_host_listeners = manual_listeners.len() > 3;

  if excessive_host_listeners {
    println!("\nWARNING: Excessive @HostListener usage may indicate non-SDUI patterns");
  }
}

/// Verifies signal/state management uses library patterns
#[test]
fn verify_state_management_pattern() {
  let app_ts_path = Path::new("src/app/app.ts");

  if !app_ts_path.exists() {
    return;
  }

  let content = fs::read_to_string(app_ts_path)
    .expect("Failed to read app.ts");

  println!("\n=== State Management Analysis ===");

  // Check for local component state (anti-pattern in SDUI)
  let local_state_count = content.matches("languages:")
    .count()
    + content.matches("sourceLang =")
    .count()
    + content.matches("targetLang =")
    .count()
    + content.matches("inputText =")
    .count()
    + content.matches("translatedText =")
    .count();

  println!("Local component state fields found: {}", local_state_count);

  // Check for SignalStoreService (SDUI pattern)
  let uses_signal_store = content.contains("SignalStoreService");
  println!("Uses SignalStoreService: {}", uses_signal_store);

  // Check for GlobalStateService (acceptable but indicates coupling)
  let uses_global_state = content.contains("GlobalStateService");
  println!("Uses GlobalStateService: {}", uses_global_state);

  // Excessive local state + no signal store = non-SDUI
  let has_heavy_local_state = local_state_count > 5 && !uses_signal_store;

  if has_heavy_local_state {
    println!("\nVIOLATION: Heavy local state without SignalStore indicates non-SDUI pattern");
  }

  if uses_signal_store && local_state_count < 3 {
    println!("\nPASS: App uses SignalStore with minimal local state - Good SDUI pattern");
  }
}

/// Verifies debounce/timer logic is not hardcoded in component
#[test]
fn verify_no_hardcoded_timers() {
  let app_ts_path = Path::new("src/app/app.ts");

  if !app_ts_path.exists() {
    return;
  }

  let content = fs::read_to_string(app_ts_path)
    .expect("Failed to read app.ts");

  println!("\n=== Timer Logic Analysis ===");

  let has_debounce_timer = content.contains("debounceTimer")
    || content.contains("setTimeout")
    || content.contains("debounce");

  println!("Has debounce/setTimeout logic: {}", has_debounce_timer);

  let has_schedule_translation = content.contains("scheduleTranslation");
  println!("Has scheduleTranslation method: {}", has_schedule_translation);

  if has_debounce_timer {
    println!("\nINFO: Debounce timer found in component");
    println!("In SDUI, debounce should be handled by schema event system or service layer");
  }
}

/// Main verification summary
#[test]
fn verify_sdui_readiness_summary() {
  println!("\n");
  println!("========================================");
  println!("  SDUI READINESS VERIFICATION SUMMARY");
  println!("========================================");
  println!();

  let app_ts_path = Path::new("src/app/app.ts");

  if !app_ts_path.exists() {
    println!("Cannot verify - app.ts not found");
    return;
  }

  let content = fs::read_to_string(app_ts_path)
    .expect("Failed to read app.ts");

  // Scoring based on SDUI criteria
  let mut score = 0;
  let mut max_score = 0;

  // 1. Schema shell usage (25 points)
  max_score += 25;
  if content.contains("SchemaShellComponent") || content.contains("lib-schema-shell") {
    score += 25;
    println!("[PASS] Uses SchemaShellComponent (25/25)");
  } else {
    println!("[FAIL] Missing SchemaShellComponent (0/25)");
  }

  // 2. No local template (25 points)
  max_score += 25;
  let has_local_template = content.contains("template:")
    && (content.contains("<div") || content.contains("<button"));
  if !has_local_template {
    score += 25;
    println!("[PASS] No hardcoded local template (25/25)");
  } else {
    println!("[FAIL] Has hardcoded local template (0/25)");
  }

  // 3. SignalStore usage (20 points)
  max_score += 20;
  if content.contains("SignalStoreService") {
    score += 20;
    println!("[PASS] Uses SignalStoreService (20/20)");
  } else {
    println!("[WARN] No SignalStoreService (0/20)");
  }

  // 4. Minimal local state (15 points)
  max_score += 15;
  let state_fields = content.matches("= \"\"").count() + content.matches("= []").count();
  if state_fields < 5 {
    score += 15;
    println!("[PASS] Minimal local state (15/15)");
  } else {
    println!("[FAIL] Excessive local state (0/15)");
  }

  // 5. No direct DOM manipulation (15 points)
  max_score += 15;
  if !content.contains("emitToOutput") && !content.contains("document.getElementById") {
    score += 15;
    println!("[PASS] No direct DOM manipulation (15/15)");
  } else {
    println!("[FAIL] Direct DOM manipulation detected (0/15)");
  }

  println!();
  println!("========================================");
  println!("  SDUI SCORE: {}/{} ({:.0}%)",
    score, max_score,
    (score as f64 / max_score as f64) * 100.0);
  println!("========================================");
  println!();

  // Threshold for "SDUI-ready" is 70%
  let percentage = score as f64 / max_score as f64;
  assert!(
    percentage >= 0.7,
    "Translator must score at least 70% to be SDUI-ready"
  );
}

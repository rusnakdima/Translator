# Prompt for Generating a Rust Project Template with Plural Folder Names, Singular File/Class Derivatives, and 2-Space Indent for Styles

## Objective

Develop a comprehensive, scientific-style template description for initializing Rust-based software projects, adaptable to Axum for web services and Tauri for desktop applications. The template must adhere to Rust's idiomatic conventions while incorporating customizations for directory structure, file naming (using plural folder names but singular folder derivatives in file and struct names, with direct files in folders), and coding practices, including a 2-size space indent tab size for style code. The output should be a standalone Markdown document titled "Rust Project Template: Standardized Architecture for Modular Applications," structured with sections including Abstract, Introduction, Project Structure, File Naming Conventions, Code Writing Standards, and Additional Best Practices. Ensure the template promotes scalability, maintainability, and robustness, aligning with SOLID principles and OOP through structs, traits, and enums. The template remains theme-agnostic, with framework-specific configurations (e.g., databases, security) defined post-template application.

## Guidelines for Template Generation

- **Prerequisites and Study**: Mandate review of Rust's documentation (The Rust Book, Cargo guide) covering modules, crates, ownership, and lifetimes. For Axum, study async handlers; for Tauri, review state management and IPC.
- **Custom Structure Modifications**: Organize files by type within `src/`, using plural folder names: `routes/`, `controllers/`, `services/`, `models/`, `helpers/`. Include optional folders like `middlewares/`, `repositories/`, or `configs/`. Use singular folder derivatives in file and struct names (e.g., `user.model.rs`, struct `UserModel`).
- **File Creation**: Prohibit automated file generation; mandate manual creation. Ensure files align with their roles (e.g., routes for request handling).
- **File Naming Conventions**: Adopt `<kebab-case-base-name>.<singular-folder-derivative>.rs`, where `<kebab-case-base-name>` derives from the PascalCase struct/trait name excluding the suffix (e.g., `user` for struct `UserModel`), and `<singular-folder-derivative>` is the singular form (e.g., `model` for `models/`).
- **Code Standards**: Enforce SOLID principles in Rust code. Use structs and traits for OOP. Ensure type safety and ownership adherence. Apply 2-size space indent for style code (e.g., CSS in Tauri).
- **Additional Best Practices**: Address error handling (`Result`, `Option`), concurrency (Tokio for Axum, async commands for Tauri), and performance. Exclude test files but recommend testing strategies.
- **Scientific Tone**: Use a formal, academic style, avoiding code snippets.
- **Output Format**: Produce a Markdown document reflecting plural folder names with singular derivatives and 2-space style indentation.

# Rust Project Template: Standardized Architecture for Modular Applications

## Abstract

This document presents a standardized template for Rust-based projects, adaptable to Axum and Tauri. It integrates Rust's conventions with customized practices, using plural folder names (e.g., `models/`, `services/`) and singular folder derivatives in file and struct names (e.g., `user.model.rs`, struct `UserModel`). Style code uses a 2-size space indent tab size. Emphasizing SOLID principles, OOP, and Rust's safety guarantees, the template ensures robustness for industrial-grade products. Theme-agnostic, it supports future configurations (e.g., databases, security).

## Introduction

### Purpose

The template establishes a robust foundation for Rust projects, ensuring consistency. Plural folder names with singular derivatives in file and struct names, plus 2-space style indentation, enhance code discoverability and consistency.

### Prerequisites

- **Documentation Review**: Study Rust's principles (modules, crates, ownership) via The Rust Book. For Axum, review async handlers; for Tauri, examine state management and IPC.
- **Tooling**: Use Cargo (`cargo new`) and Rust Analyzer. Manually create files.
- **Assumptions**: Projects start with Cargo's structure (`src/main.rs`, `Cargo.toml`), then customized. No test files.

## Project Structure

The structure uses plural folder names within `src/`, with singular derivatives in file and struct names.

- **Core Subfolders**:
  - `routes/`: For API endpoints (Axum) or command handlers (Tauri).
  - `controllers/`: For orchestration logic.
  - `services/`: For business logic.
  - `models/`: For data structures (e.g., structs for database records).
  - `helpers/`: For utility functions or modules.
- **Optional Folders**: Include `middlewares/`, `repositories/`, or `configs/` as needed.
- **Rationale**: Plural folder names (e.g., `services/`) align with conventions, while singular derivatives (e.g., `service`) ensure clarity.

### Module Path Management

Use `mod.rs` files for public re-exports (e.g., `pub mod user_model;` in `models/mod.rs`), enabling concise imports (e.g., `use crate::models::user_model;`). Keep paths flat to reduce compilation times.

## File Naming and Organization Conventions

File naming uses singular folder derivatives for plural folder names.

### General Rules

- File names: `<kebab-case-base-name>.<singular-folder-derivative>.rs`, where `<kebab-case-base-name>` is the base kebab-case name from the PascalCase struct/trait name excluding the suffix (e.g., `user` for struct `UserModel`), and `<singular-folder-derivative>` is the singular form (e.g., `model` for `models/`).
- Manually create files; avoid automated generation.
- Place files directly in folders (e.g., `services/auth.service.rs`). Use submodules for grouping.
- Exclude test files.

### Specific Examples

- `models/user.model.rs` (struct: `UserModel`).
- `services/auth.service.rs` (struct: `AuthService`).
- `controllers/user.controller.rs` (struct: `UserController`).
- `routes/api.route.rs` (struct: `ApiRoute`).
- `helpers/logger.helper.rs` (trait: `LoggerHelper`).

## Code Writing Standards

### Rust and SOLID Principles

- **Single Responsibility**: Each file handles one concern.
- **Open-Closed**: Use traits for extensibility.
- **Liskov Substitution**: Ensure structs implementing traits are substitutable.
- **Interface Segregation**: Define granular traits in `models/`.
- **Dependency Inversion**: Inject dependencies via structs or trait objects.
- **OOP Integration**: Use structs with impl blocks and traits for polymorphism.
- **Type Safety**: Avoid unsafe code unless justified. Use generics.
- **Error Handling**: Use `Result` and `Option`; propagate errors with `?`.
- **Concurrency**: Use Tokio for Axum; leverage Tauri's async commands.
- **Style Indentation**: For style code (e.g., CSS in Tauri applications), apply a 2-size space indent tab size in style definitions.
- **Naming Conventions**: All variables and functions in .rs files must be written in camelCase (e.g., `userName`, `calculateTotal`).

## Additional Best Practices

- **Performance**: Favor stack allocation and enums. Minimize cloning with references.
- **Error Handling**: Define custom error types in `helpers/`.
- **Modularity**: Structure code as reusable modules.
- **Testing Strategy**: Recommend unit testing with `#[test]` and integration testing with `cargo test`.
- **Framework Adaptability**: Design for Axum's async ecosystem or Tauri's IPC.

This template ensures a scalable Rust project structure, using plural folder names with singular derivatives and 2-space style indentation, ready for business needs with Axum or Tauri.

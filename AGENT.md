# TranslationAssistant Project Guidelines

This project follows strict coding conventions for both Rust (Tauri backend) and Angular (frontend). All development must adhere to the guidelines defined in the reference documents.

## Reference Documents

- **Rust Guidelines**: [prompt_for_rust.md](./prompt_for_rust.md)
  - Project structure with plural folder names and singular file derivatives
  - File naming: `<kebab-case-base-name>.<singular-folder-derivative>.rs`
  - All variables and functions in camelCase
  - SOLID principles and OOP with structs/traits
  - 2-space indent for style code

- **Angular Guidelines**: [prompt_for_angular.md](./prompt_for_angular.md)
  - Project structure with plural folder names (components/, views/, models/, services/, helpers/)
  - File naming: `<kebab-case-base-name>.<singular-folder-derivative>.ts`
  - Components in subfolders: `components/<name>/<name>.component.ts`
  - TailwindCSS class ordering with utility prefixes (md:, dark:)
  - 2-space indent for style code
  - Modern Angular features: signals, @if/@for control flow

## Project Structure

```
TranslationAssistant/
├── src/
│   ├── app/
│   │   ├── components/       # Reusable UI elements
│   │   ├── views/           # Page-like structures
│   │   ├── models/          # Data structures
│   │   ├── services/        # Business logic
│   │   └── helpers/         # Utilities
│   ├── assets/
│   └── styles.css
├── src-tauri/
│   ├── src/
│   │   ├── routes/          # Command handlers
│   │   ├── services/        # Business logic
│   │   ├── models/          # Data structures
│   │   └── helpers/         # Utilities
│   └── Cargo.toml
└── libs/
    └── trans/               # Translation library
```

## Key Conventions

### Rust (Backend)
- Variables and functions: `camelCase`
- Files: `kebab-case-name.singular-folder-derivative.rs`
- Structs: `PascalCase`
- Traits: `PascalCase`

### Angular (Frontend)
- Components: `<name>.component.ts` with `NameComponent` class
- Services: `<name>.service.ts` with `NameService` class
- Models: `<name>.model.ts` with `Name` class/interface
- HTML templates: 2-space indent for TailwindCSS classes

## Development Workflow

1. **Backend (Rust/Tauri)**:
   - Implement business logic in `services/`
   - Define data models in `models/`
   - Create Tauri commands in `routes/`
   - Use camelCase for all variables and functions

2. **Frontend (Angular)**:
   - Create components in `components/<name>/`
   - Implement services in `services/`
   - Define models in `models/`
   - Use signals for reactive state
   - Follow TailwindCSS class ordering

## Important Notes

- Always read reference documents before making changes
- Do not use Angular CLI generators (`ng generate`)
- Manually create all files to enforce standards
- Run `cargo check` before committing Rust code
- Run `npm run build` before committing Angular code

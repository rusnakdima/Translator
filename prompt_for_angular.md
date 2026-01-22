# Prompt for Generating an Angular Project Template with Plural Folder Names, Singular File/Class Derivatives, and 2-Space Indent for Styles

## Objective

Develop a comprehensive, scientific-style template description for initializing Angular-based frontend applications, suitable for enterprise-level, business-critical products. The template must adhere to Angular's official standards while incorporating customizations for directory structure, file naming (using plural folder names but singular folder derivatives in file and class names, with subfolders for components and views containing only the base name), and coding practices, including a 2-size space indent tab size for style code (e.g., in HTML templates with TailwindCSS). The output should be a standalone Markdown document titled "Angular Project Template: Standardized Setup for Enterprise-Level Development," structured with sections including Abstract, Introduction, Project Structure, File Naming Conventions, Code Writing Standards, and Additional Best Practices. Ensure the template promotes scalability, maintainability, and robustness, aligning with SOLID principles and modern Angular features like signals and control flow. The template remains theme-agnostic, serving as a reusable foundation for future project specifications, with integrations like TailwindCSS v4 and Angular Material described narratively.

## Guidelines for Template Generation

- **Prerequisites and Study**: Require developers to study the official Angular documentation (version 20) for standalone components, signals, control flow (`@if`, `@for`), and routing. Include guidance on standard directory layouts, TypeScript conventions, and third-party integrations like TailwindCSS v4 (utility-first styling with prefixes like `md:`, `dark:`) and Angular Material for UI components.
- **Custom Structure Modifications**: Organize files by type within `src/app/`, using plural folder names: `components/` (for UI elements), `views/` (for page structures), `models/` (for data structures), `services/` (for business logic), `helpers/` (for utilities), `guards/` (for route guards). For `components/` and `views/`, use subfolders named `<kebab-case-base-name>/` (e.g., `user/`) containing `<kebab-case-base-name>.<singular-folder-derivative>.ts` and `.html` (e.g., `user.component.ts` and `user.component.html`). Use singular folder derivatives in file and class names (e.g., `user.component.ts`, class `UserComponent`).
- **File Creation**: Prohibit use of Angular CLI for generating files (e.g., `ng generate`); mandate manual creation to enforce custom standards. Ensure files align with their intended Angular roles (e.g., components for UI, services for logic).
- **File Naming Conventions**: Adopt a format of `<kebab-case-base-name>.<singular-folder-derivative>.ts` or `.html`, where `<kebab-case-base-name>` is the base name from the PascalCase class name excluding the suffix (e.g., `user` for class `UserComponent`), and `<singular-folder-derivative>` is the singular form of the plural folder name (e.g., `component` for `components/`, `service` for `services/`). Class names combine base name with singular derivative (e.g., `UserComponent`).
- **Code Standards**: Enforce SOLID principles in TypeScript code. Utilize Angular decorators, interfaces, and signals for type safety and reactivity. Ensure style code (e.g., TailwindCSS classes in HTML) uses a 2-size space indent tab size.
- **TailwindCSS Class Ordering**: Enforce a strict order for TailwindCSS classes within HTML `class` attributes to improve readability and consistency. For classes with utility prefixes (e.g., `md:`, `dark:`, `min-[...]:`), place them immediately after their non-prefixed counterparts within the same category, if present; otherwise, place them according to their category in the sequence below. Group and sort classes into these categories in this sequence, preserving the original order of classes within each category unless specified:
  1. **Position types**: Classes like `static`, `absolute`, `relative`, `fixed`, `sticky`, followed by their utility-prefixed versions (e.g., `md:static`, `dark:absolute`).
  2. **Display types**: Classes like `flex`, `block`, `grid`, `inline`, `inline-block`, `hidden`, `inline-flex`, `inline-grid`, followed by their utility-prefixed versions (e.g., `md:flex`, `dark:grid`).
  3. **Flex/grid specifics** (conditional):
     - If `flex` or `inline-flex` is present in category 2, include flex-related classes (e.g., `flex-col`, `flex-row`, `flex-wrap`, `flex-nowrap`) followed by their utility-prefixed versions (e.g., `md:flex-col`, `dark:flex-wrap`).
     - If `grid` or `inline-grid` is present in category 2, include grid-related classes (e.g., `grid-cols-*`, `grid-rows-*`, `grid-flow-*`) followed by their utility-prefixed versions (e.g., `md:grid-cols-2`, `dark:grid-flow-row`).
  4. **Alignment/positioning**: Classes like `justify-*` (e.g., `justify-center`, `justify-between`), `items-*` (e.g., `items-center`, `items-start`), `content-*`, `self-*`, `place-*`, followed by their utility-prefixed versions (e.g., `md:justify-center`, `dark:items-start`).
  5. **Gap/spacing**: Classes like `gap-*`, `space-x-*`, `space-y-*`, followed by their utility-prefixed versions (e.g., `md:gap-4`, `dark:space-x-2`).
  6. **Coordinate positions**: Classes like `left-*`, `right-*`, `top-*`, `bottom-*`, `inset-*`, followed by their utility-prefixed versions (e.g., `md:left-4`, `dark:top-0`).
  7. **Margins, padding, z-index**: Classes like `m-*` (margins, e.g., `m-4`, `mx-2`, `my-auto`), `p-*` (padding, e.g., `p-4`, `px-2`, `py-auto`), `z-*` (e.g., `z-10`), followed by their utility-prefixed versions (e.g., `md:m-2`, `dark:p-4`).
  8. **Borders** (in this sub-order):
     - Rounded corners: `rounded-*` (e.g., `rounded-md`, `rounded-t-lg`), followed by utility-prefixed versions (e.g., `md:rounded-md`).
     - Border sides: `border` (all sides), `border-t`, `border-r`, `border-b`, `border-l`, `border-x`, `border-y`, followed by utility-prefixed versions (e.g., `md:border-t`).
     - Border width: `border-*` where \* is a width value (e.g., `border-2`, `border-t-4`), followed by utility-prefixed versions (e.g., `md:border-2`).
     - Border color: `border-*` where \* is a color (e.g., `border-gray-500`), followed by utility-prefixed versions (e.g., `md:border-gray-500`).
  9. **Background colors**: Classes like `bg-*` (e.g., `bg-blue-500`, `bg-gradient-*`), followed by their utility-prefixed versions (e.g., `md:bg-blue-500`).
  10. **Sizes**: Classes for width and height like `w-*` (e.g., `w-full`, `w-1/2`), `h-*` (e.g., `h-screen`, `h-64`), `min-w-*`, `max-w-*`, `min-h-*`, `max-h-*`, followed by their utility-prefixed versions (e.g., `md:w-full`, `dark:h-64`).
  11. **Text styles** (in this sub-order):
      - Font properties: `font-*` (e.g., `font-bold`, `font-sans`, `font-mono`), followed by utility-prefixed versions (e.g., `md:font-bold`).
      - Text size: `text-*` where \* is a size (e.g., `text-lg`, `text-2xl`), followed by utility-prefixed versions (e.g., `md:text-lg`).
      - Text color: `text-*` where \* is a color (e.g., `text-red-600`), followed by utility-prefixed versions (e.g., `md:text-red-600`).
  12. **Miscellaneous/others**: All remaining Tailwind classes not covered above, such as shadows (`shadow-*`), cursors (`cursor-*`), transitions (`duration-*`, `ease-*`), transforms (`transform`, `scale-*`, `rotate-*`, `translate-*`), overflows (`overflow-*`), opacities (`opacity-*`), etc., followed by their utility-prefixed versions (e.g., `md:shadow-md`, `dark:cursor-pointer`). Place these in their original relative order within this category.
      Utility Prefix Rule: For each category, non-prefixed classes (e.g., `flex`) come first, followed by their utility-prefixed versions (e.g., `md:flex`, `dark:flex`) in the order they appear in the original class list. If no non-prefixed class exists for a category, utility-prefixed classes are placed according to the category sequence. Non-Tailwind or custom classes are placed at the end, after category 12, in their original relative order.
- **Additional Best Practices**: Address error handling (RxJS operators), environment configuration (`environment.ts`), performance (AOT, tree-shaking, signals), security (HttpClient interceptors), and modularity (lazy loading). Exclude test files but recommend testing strategies. Optimize with modern features like `@defer`.
- **Scientific Tone**: Use a formal, academic style, avoiding code snippets.
- **Output Format**: Produce a Markdown document with a clear, hierarchical structure, reflecting plural folder names with singular derivatives (base name only in subfolders/files) and 2-space style indentation.

# Angular Project Template: Standardized Setup for Enterprise-Level Development

## Abstract

This document defines a standardized template for Angular version 20 projects, integrated with TailwindCSS version 4 and Angular Material, designed for scalable, enterprise-grade applications. It customizes Angular's conventions by organizing files by type with plural folder names (e.g., `components/`, `models/`) and using singular folder derivatives in file and class names (e.g., `user.component.ts`, class `UserComponent`). Style code uses a 2-size space indent tab size. TailwindCSS classes, including utility-prefixed variants (e.g., `md:`, `dark:`), are ordered systematically to ensure consistency. Emphasizing SOLID principles and modern Angular features, the template ensures robustness and maintainability for business-critical systems. Theme-agnostic, it serves as a reusable foundation adaptable to diverse project requirements.

## Introduction

### Purpose

The template establishes a consistent, scalable architecture for Angular projects, minimizing technical debt and ensuring production readiness. By requiring study of Angular v20, TailwindCSS v4, and Angular Material documentation, developers align with modern APIs and best practices. The use of plural folder names with singular derivatives, a 2-space indent for styles, and ordered TailwindCSS classes (including utility prefixes) enhances code discoverability and consistency.

### Prerequisites

- **Documentation Review**: Study Angular v20 documentation for standalone components, signals, and control flow syntax (`@if`, `@for`). Review TailwindCSS v4 for utility-first styling, including utility prefixes (e.g., `md:`, `dark:`, `min-[...]:`), and Angular Material for component integration and theming.
- **Tooling**: Use Node.js (compatible with Angular v20), Angular CLI for initial setup (`ng new`), and a TypeScript-aware IDE. Manually create files to enforce standards.
- **Assumptions**: Projects start with Angular CLI's default structure (`src/app/`), then customized. No test or style files are created; styling relies on TailwindCSS utilities.

## Project Structure

The project adopts a type-based organization within `src/app/`, using plural folder names, with singular derivatives in file and class names for consistency.

- **Core Subfolders**:
  - `components/`: For reusable UI elements with templates and logic.
  - `views/`: For page-like structures composing components (e.g., smart components).
  - `models/`: For TypeScript interfaces and classes defining data structures.
  - `services/`: For injectable classes handling business logic or API interactions.
  - `helpers/`: For utility functions or stateless classes.
  - `guards/`: For route guards implementing navigation or authorization logic.
- **Rationale**: Plural folder names (e.g., `components/`) align with Angular's conventional naming, while singular derivatives (e.g., `component`) ensure clarity. Type-based grouping simplifies file lookup in large projects.

### Path Configuration

Configure `tsconfig.json` with path aliases:

- `@components/*` for `src/app/components/*`.
- `@views/*` for `src/app/views/*`.
- `@models/*` for `src/app/models/*`.
- `@services/*` for `src/app/services/*`.
- `@helpers/*` for `src/app/helpers/*`.
- `@guards/*` for `src/app/guards/*`.
  Use aliases consistently to shorten imports and enhance readability.

## File Naming and Organization Conventions

File naming extends Angular's kebab-case standard, using singular folder derivatives for plural folder names.

### General Rules

- File names: `<kebab-case-base-name>.<singular-folder-derivative>.ts` or `.html`, where `<kebab-case-base-name>` is the base kebab-case name from the PascalCase class name excluding the suffix (e.g., `user` for class `UserComponent`), and `<singular-folder-derivative>` is the singular form of the folder name (e.g., `component` for `components/`, `service` for `services/`).
- Manually create files; avoid CLI generation.
- For `components/` and `views/`: Use subfolders named `<kebab-case-base-name>/` containing `<kebab-case-base-name>.<singular-folder-derivative>.ts` and `.html` (or `.view.ts` and `.html` for views).
- For `models/`, `services/`, `helpers/`, `guards/`: Place `.ts` files directly in the folder, without subfolders unless logically grouped.
- Exclude test files (`.spec.ts`) and style files; use TailwindCSS utilities in HTML.

### Specific Examples

- `components/user/user.component.ts` (class: `UserComponent`).
- `components/user/user.component.html`.
- `views/dashboard/dashboard.view.ts` (class: `DashboardView`).
- `views/dashboard/dashboard.view.html`.
- `models/user.model.ts` (interface/class: `User`).
- `services/auth.service.ts` (class: `AuthService`).
- `guards/role.guard.ts` (class: `RoleGuard`).

This format ensures traceability and supports IDE navigation.

## Code Writing Standards

### TypeScript and SOLID Principles

- **Single Responsibility**: Each file handles one concern (e.g., components render UI, services manage logic).
- **Open-Closed**: Use interfaces and standalone components for extensibility.
- **Liskov Substitution**: Ensure substitutable types via interfaces in `models/`.
- **Interface Segregation**: Define focused interfaces for specific consumers.
- **Dependency Inversion**: Inject services via constructor injection with `@Injectable()`.
- **Type Safety**: Use interfaces in `models/`; avoid `any`. Leverage generics for flexibility.
- **Modern Features**: Use Angular v20 signals for reactive state, `@if`/`@for` for control flow, and `@defer` for lazy loading.

### HTML Templates

- Use Angular v20 control flow (`@if`, `@for`, `@switch`) instead of `*ngIf`, `*ngFor`.
- Integrate Angular Material components (e.g., `mat-button`, `mat-card`) with TailwindCSS utilities for styling.
- **Style Indentation**: Apply a 2-size space indent tab size for TailwindCSS utility classes in HTML templates (e.g., align nested classes with two spaces in multi-line attribute declarations).
- **TailwindCSS Class Ordering**: Organize classes in `class` attributes following this sequence for consistency and maintainability:
  1. Position types (e.g., `static`, `absolute`, `relative`), followed by utility-prefixed versions (e.g., `md:static`, `dark:absolute`).
  2. Display types (e.g., `flex`, `block`, `grid`, `hidden`), followed by utility-prefixed versions (e.g., `md:flex`, `dark:grid`).
  3. If `flex` or `inline-flex` is present, include flex-specific classes (e.g., `flex-col`, `flex-row`, `flex-wrap`), followed by utility-prefixed versions (e.g., `md:flex-col`); if `grid` or `inline-grid`, include grid-specific classes (e.g., `grid-cols-*`), followed by utility-prefixed versions (e.g., `md:grid-cols-2`).
  4. Alignment (e.g., `justify-*`, `items-*`), followed by utility-prefixed versions (e.g., `md:justify-center`).
  5. Gaps (e.g., `gap-*`), followed by utility-prefixed versions (e.g., `md:gap-4`).
  6. Coordinates (e.g., `top-*`, `left-*`), followed by utility-prefixed versions (e.g., `md:left-4`).
  7. Margins/padding/z-index (e.g., `m-*`, `p-*`, `z-*`), followed by utility-prefixed versions (e.g., `md:m-2`).
  8. Borders: rounded (e.g., `rounded-*`), sides (e.g., `border-t`), width (e.g., `border-2`), color (e.g., `border-gray-500`), each followed by utility-prefixed versions (e.g., `md:rounded-md`, `dark:border-gray-500`).
  9. Backgrounds (e.g., `bg-*`), followed by utility-prefixed versions (e.g., `md:bg-blue-500`).
  10. Sizes (e.g., `w-*`, `h-*`), followed by utility-prefixed versions (e.g., `md:w-full`).
  11. Text styles: font (e.g., `font-*`), size (e.g., `text-lg`), color (e.g., `text-red-600`), each followed by utility-prefixed versions (e.g., `md:text-lg`).
  12. Other classes (e.g., `shadow-*`, `cursor-*`, `duration-*`, `transform`), followed by utility-prefixed versions (e.g., `md:shadow-md`).
      Non-Tailwind or custom classes are placed last, in their original order.
- Ensure accessibility with ARIA attributes and semantic HTML.
- Keep templates declarative; offload logic to TypeScript.

## Additional Best Practices

- **Modularity**: Support lazy-loaded modules for routing, using `loadChildren`.
- **Error Handling**: Implement robust error management in services with RxJS operators.
- **Environment Configuration**: Use `environment.ts` for managing API URLs and settings.
- **Performance**: Enable AOT compilation and tree-shaking in `angular.json`. Use signals to optimize change detection.
- **Security**: Sanitize inputs and use HttpClient with interceptors for API security.
- **API Documentation**: Prepare for OpenAPI integration if RESTful APIs are added later.
- **Testing Strategy**: Recommend Jasmine/Karma for unit tests and Protractor for e2e in future implementations.

This template ensures a professional, scalable Angular project structure, using plural folder names with singular derivatives in file and class names, a 2-space indent for style code, and systematically ordered TailwindCSS classes including utility prefixes, ready for adaptation to specific business needs.

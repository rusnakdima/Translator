import angular from "@angular-eslint/eslint-plugin";
import angularTemplate from "@angular-eslint/eslint-plugin-template";
import angularTemplateParser from "@angular-eslint/template-parser";
import tseslint from "typescript-eslint";
import js from "@eslint/js";
import globals from "globals";

export default [
  {
    ignores: ["**/*.spec.ts", "**/test.ts", "**/polyfills.ts"],
  },
  {
    files: ["**/*.ts"],
    plugins: {
      "@angular-eslint": angular,
    },
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        project: "./tsconfig.json",
        createDefaultProgram: true,
      },
      globals: {
        ...globals.browser,
        ...globals.es2020,
      },
    },
    rules: {
      ...angular.configs.recommended.rules,
      "@angular-eslint/directive-selector": [
        "error",
        { type: "attribute", prefix: "app", style: "camelCase" },
      ],
      "@angular-eslint/component-selector": [
        "error",
        { type: "element", prefix: "app", style: "kebab-case" },
      ],
    },
  },
  {
    files: ["**/*.html"],
    plugins: {
      "@angular-eslint/template": angularTemplate,
    },
    languageOptions: {
      parser: angularTemplateParser,
    },
    rules: {
      ...angularTemplate.configs.recommended.rules,
    },
  },
];

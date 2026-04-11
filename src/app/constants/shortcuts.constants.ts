export interface Shortcut {
  key: string;
  description: string;
  action: string;
}

export const SHORTCUTS: Shortcut[] = [
  {
    key: "Ctrl + /",
    description: "Show keyboard shortcuts",
    action: "show-shortcuts",
  },
  {
    key: "Ctrl + Enter",
    description: "Translate text",
    action: "translate",
  },
  {
    key: "Ctrl + L",
    description: "Swap languages",
    action: "swap",
  },
  {
    key: "Ctrl + Shift + V",
    description: "Quick paste to source",
    action: "quick-paste",
  },
  {
    key: "Ctrl + Shift + C",
    description: "Quick copy translation",
    action: "quick-copy",
  },
  {
    key: "Ctrl + Shift + S",
    description: "Focus source input",
    action: "focus-source",
  },
  {
    key: "Ctrl + Shift + L",
    description: "Focus source language",
    action: "focus-source-lang",
  },
  {
    key: "Ctrl + Shift + ;",
    description: "Focus target language",
    action: "focus-target-lang",
  },
  {
    key: "Escape",
    description: "Close this overlay",
    action: "close",
  },
];

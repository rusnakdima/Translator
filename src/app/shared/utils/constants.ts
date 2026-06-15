export type ToastKind = "success" | "error" | "info";

export const ToastKind = {
  Success: "success" as ToastKind,
  Error: "error" as ToastKind,
  Info: "info" as ToastKind,
};

export interface Shortcut {
  keys: string;
  description: string;
}

export const SHORTCUTS: Shortcut[] = [
  { keys: "Ctrl + Enter", description: "Translate" },
  { keys: "Ctrl + S", description: "Swap languages" },
  { keys: "Ctrl + 1-9", description: "Select target language" },
  { keys: "?", description: "Show shortcuts" },
];

import { onMount, onDestroy } from "svelte";

export interface Shortcut {
  key: string;
  modifiers?: string[];
  label?: string;
  description?: string;
  handler?: string;
  id?: string;
  enabled?: boolean;
  category?: string;
}

const shortcuts: Shortcut[] = [
  { key: "Enter", modifiers: ["ctrl"], label: "Translate" },
  { key: "S", modifiers: ["ctrl", "shift"], label: "Swap languages" },
  { key: "K", modifiers: ["ctrl"], label: "Clear text" },
  { key: "C", modifiers: ["ctrl", "shift"], label: "Copy translation" },
  { key: ",", modifiers: ["ctrl"], label: "Open settings" },
  { key: "?", label: "Show shortcuts" },
];

let keydownHandler: ((event: KeyboardEvent) => void) | null = null;

export function initShortcuts(customHandler?: (action: string) => void): void {
  if (keydownHandler) {
    document.removeEventListener("keydown", keydownHandler);
  }

  keydownHandler = (event: KeyboardEvent) => {
    if (event.key === "F1") {
      event.preventDefault();
      window.dispatchEvent(new CustomEvent("onShortcutsOpen"));
      return;
    }

    const ctrl = event.ctrlKey || event.metaKey;
    const shift = event.shiftKey;

    if (event.key === "Enter" && ctrl) {
      event.preventDefault();
      customHandler?.("translate");
    } else if (event.key === "S" && ctrl && shift) {
      event.preventDefault();
      customHandler?.("swap");
    } else if (event.key === "K" && ctrl) {
      event.preventDefault();
      customHandler?.("clear");
    } else if (event.key === "C" && ctrl && shift) {
      event.preventDefault();
      customHandler?.("copy");
    }
  };

  document.addEventListener("keydown", keydownHandler);
}

export function getShortcuts(): Shortcut[] {
  return [...shortcuts];
}

export function cleanupShortcuts(): void {
  if (keydownHandler) {
    document.removeEventListener("keydown", keydownHandler);
    keydownHandler = null;
  }
}

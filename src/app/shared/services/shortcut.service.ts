import { Injectable } from "@angular/core";

export interface Shortcut {
  key: string;
  handler?: string;
  description?: string;
  id?: string;
  modifiers?: string[];
  label?: string;
  category?: string;
  enabled?: boolean;
}

@Injectable({ providedIn: "root" })
export class ShortcutService {
  private shortcuts: Shortcut[] = [];

  init(): void {
    this.shortcuts = [
      { key: "Enter", modifiers: ["ctrl"], label: "Translate" },
      { key: "S", modifiers: ["ctrl", "shift"], label: "Swap languages" },
      { key: "K", modifiers: ["ctrl"], label: "Clear text" },
      { key: "C", modifiers: ["ctrl", "shift"], label: "Copy translation" },
      { key: ",", modifiers: ["ctrl"], label: "Open settings" },
      { key: "?", label: "Show shortcuts" },
    ];
    this.registerKeyboardListeners();
  }

  private registerKeyboardListeners(): void {
    document.removeEventListener("keydown", this.handleKeyDown);
    document.addEventListener("keydown", this.handleKeyDown.bind(this));
  }

  private handleKeyDown(event: KeyboardEvent): void {
    if (event.key === "F1") {
      event.preventDefault();
      window.dispatchEvent(new CustomEvent("onShortcutsOpen"));
    }
  }

  getShortcuts(): Shortcut[] {
    return [...this.shortcuts];
  }
}

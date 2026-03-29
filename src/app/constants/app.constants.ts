import type { ToastType } from "@components/toast/toast.component";

/** Tauri `Window::emit` / `listen` event names */
export const TAURI_EVENTS = {
  translationResult: "translation-result",
} as const;

/** Serialized `Response.status` from the Rust API */
export const RESPONSE_STATUS = {
  error: "error",
  success: "success",
} as const;

export const ToastKind = {
  Info: "info",
  Success: "success",
  Error: "error",
} as const satisfies Record<string, ToastType>;

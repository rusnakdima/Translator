import type { ToastType } from "@components/toast/toast.component";

declare global {
  interface Window {
    showToast?: (message: string, type?: ToastType, duration?: number) => void;
  }
}

export {};

import { ToastType } from "@shared/utils/constants";

declare global {
  interface Window {
    showToast: (message: string, type?: ToastType, duration?: number) => void;
  }
}

export {};

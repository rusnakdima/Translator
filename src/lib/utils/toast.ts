import { ToastKind, type ToastType } from "./constants";

export class ToastHelper {
  private static toastService: { info: (msg: string, opts?: { duration?: number }) => void; success: (msg: string, opts?: { duration?: number }) => void; error: (msg: string, opts?: { duration?: number }) => void } | null = null;

  static setToastService(service: typeof ToastHelper.prototype.toastService) {
    ToastHelper.toastService = service;
  }

  static show(
    message: string,
    type: ToastType = ToastKind.Info,
    duration: number = 3000,
  ): void {
    if (ToastHelper.toastService) {
      const svc = ToastHelper.toastService;
      if (type === "success") svc.success(message, { duration });
      else if (type === "error") svc.error(message, { duration });
      else svc.info(message, { duration });
    } else {
      (window as Window & { showToast?: (msg: string, type: ToastType) => void }).showToast?.(message, type);
    }
  }
}

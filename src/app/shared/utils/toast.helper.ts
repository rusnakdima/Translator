import { ToastService } from "@tauri-front/shared";
import { ToastType, ToastKind } from "@shared/utils/constants";

export class ToastHelper {
  private static toastService: ToastService | null = null;

  static setToastService(service: ToastService) {
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
      window.showToast?.(message, type);
    }
  }
}

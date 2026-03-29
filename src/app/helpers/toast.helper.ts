import { ToastType } from "@components/toast/toast.component";
import { ToastKind } from "@constants/app.constants";

export class ToastHelper {
  static show(
    message: string,
    type: ToastType = ToastKind.Info,
    duration: number = 3000,
  ): void {
    const showToast = window.showToast;
    if (!showToast) {
      console.warn("Toast function not found");
      return;
    }
    showToast(message, type, duration);
  }
}

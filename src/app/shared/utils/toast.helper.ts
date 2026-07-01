import { EventBusService } from "@tauri-front/shared";
import { ToastType, ToastKind } from "@shared/utils/constants";

export class ToastHelper {
  private static eventBus: EventBusService | null = null;

  static setEventBus(bus: EventBusService) {
    ToastHelper.eventBus = bus;
  }

  static show(
    message: string,
    type: ToastType = ToastKind.Info,
    duration: number = 3000,
  ): void {
    if (ToastHelper.eventBus) {
      ToastHelper.eventBus.showToast(message, type as any, duration);
    } else {
      window.showToast?.(message, type as any, duration);
    }
  }
}

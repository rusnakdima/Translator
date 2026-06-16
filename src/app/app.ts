/* sys lib */
import { Component, signal } from "@angular/core";

/* components */
import { TranslationComponent } from "@features/translation/views/translation/translation.component";
import { ToastComponent } from "@components/toast/toast.component";
import { ToastKind } from "@shared/utils/constants";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [TranslationComponent, ToastComponent],
  templateUrl: "./app.html",
})
export class App {
  toastMessage = signal<string>("");
  toastVisible = signal<boolean>(false);
  toastType = signal<ToastKind>(ToastKind.Info);

  constructor() {
    window.showToast = (
      message: string,
      type: ToastKind = ToastKind.Info,
      duration: number = 3000,
    ) => {
      this.toastMessage.set(message);
      this.toastType.set(type);
      this.toastVisible.set(true);
      setTimeout(() => {
        this.toastVisible.set(false);
      }, duration);
    };
  }
}

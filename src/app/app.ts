/* sys lib */
import { Component, signal } from "@angular/core";

/* pages */
import { TranslationPage } from "@app/pages/translation/translation.page";

/* components */
import { ToastComponent } from "@components/toast/toast.component";
import { ToastType, ToastKind } from "@shared/utils/constants";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [TranslationPage, ToastComponent],
  templateUrl: "./app.html",
})
export class App {
  toastMessage = signal<string>("");
  toastVisible = signal<boolean>(false);
  toastType = signal<ToastType>(ToastKind.Info);

  constructor() {
    window.showToast = (
      message: string,
      type: ToastType = ToastKind.Info,
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

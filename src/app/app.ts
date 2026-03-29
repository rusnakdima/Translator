/* sys lib */
import { Component, signal } from "@angular/core";

/* components */
import { TranslationComponent } from "@views/translation/translation.component";
import { ToastComponent, ToastType } from "@components/toast/toast.component";
import { ToastKind } from "@constants/app.constants";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [TranslationComponent, ToastComponent],
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

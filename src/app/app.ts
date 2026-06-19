/* sys lib */
import { Component, signal } from "@angular/core";

/* views */
import { TranslationView } from "@app/views/translation-view/translation-view.view";

/* components */
import { ToastComponent } from "@components/toast.component/toast.component";
import { ToastType, ToastKind } from "@shared/utils/constants";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [TranslationView, ToastComponent],
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

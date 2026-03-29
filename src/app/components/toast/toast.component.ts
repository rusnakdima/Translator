/* sys lib */
import { Component, input, computed } from "@angular/core";

export type ToastType = "success" | "error" | "info";

@Component({
  selector: "app-toast",
  standalone: true,
  templateUrl: "./toast.component.html",
})
export class ToastComponent {
  message = input.required<string>();
  isVisible = input.required<boolean>();
  type = input<ToastType>("info");

  toastClass = computed(() => {
    const baseClass =
      "fixed bottom-4 right-4 z-50 translate-y-0 opacity-100 transition-all duration-300 rounded-xl px-6 py-3 shadow-lg border";
    const typeClasses = {
      success:
        "dark:bg-green-800 dark:border-green-600 dark:text-green-100 bg-green-50 border-green-400 text-green-800",
      error:
        "dark:bg-red-800 dark:border-red-600 dark:text-red-100 bg-red-50 border-red-400 text-red-800",
      info: "dark:bg-slate-800 dark:border-slate-600 dark:text-slate-100 bg-white border-slate-300 text-slate-700",
    };
    return `${baseClass} ${typeClasses[this.type()]}`;
  });
}

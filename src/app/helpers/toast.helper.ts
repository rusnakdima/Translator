export class ToastHelper {
  private static toastElement: HTMLElement | null = null;
  private static messageElement: HTMLElement | null = null;

  static initialize(): void {
    this.toastElement = document.getElementById("toast");
    this.messageElement = document.getElementById("toastMessage");
  }

  static show(message: string, duration: number = 3000): void {
    if (!this.toastElement || !this.messageElement) {
      console.warn("Toast elements not found");
      return;
    }

    this.messageElement.textContent = message;
    this.toastElement.classList.remove("translate-y-20", "opacity-0");

    setTimeout(() => {
      this.hide();
    }, duration);
  }

  static hide(): void {
    if (!this.toastElement) {
      return;
    }

    this.toastElement.classList.add("translate-y-20", "opacity-0");
  }
}

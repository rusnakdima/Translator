/* sys lib */
import { Component, input, output } from "@angular/core";

/* constants */
import { SHORTCUTS, Shortcut } from "@constants/shortcuts.constants";

/* components */
import { AppIconComponent } from "@components/icons/app-icon.component";

@Component({
  selector: "app-shortcuts-overlay",
  standalone: true,
  imports: [AppIconComponent],
  templateUrl: "./shortcuts-overlay.component.html",
})
export class ShortcutsOverlayComponent {
  shortcuts = input.required<boolean>();
  close = output<void>();

  shortcutsList: Shortcut[] = SHORTCUTS;

  onClose(): void {
    this.close.emit();
  }

  onBackdropClick(event: MouseEvent): void {
    if ((event.target as HTMLElement).classList.contains("overlay-backdrop")) {
      this.onClose();
    }
  }
}

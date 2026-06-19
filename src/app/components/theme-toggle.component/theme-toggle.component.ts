/* sys lib */
import { Component, input, output } from "@angular/core";

@Component({
  selector: "app-theme-toggle",
  standalone: true,
  templateUrl: "./theme-toggle.component.html",
})
export class ThemeToggleComponent {
  isDark = input.required<boolean>();
  toggleChange = output<void>();
}

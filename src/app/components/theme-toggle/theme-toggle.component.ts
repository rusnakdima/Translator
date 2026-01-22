/* sys lib */
import { Component, input, output } from "@angular/core";
import { CommonModule } from "@angular/common";

@Component({
  selector: "app-theme-toggle",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./theme-toggle.component.html",
})
export class ThemeToggleComponent {
  isDark = input.required<boolean>();
  toggle = output<void>();
}

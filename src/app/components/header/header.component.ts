/* sys lib */
import { Component, input, output } from "@angular/core";
import { CommonModule } from "@angular/common";

/* components */
import { ThemeToggleComponent } from "@components/theme-toggle/theme-toggle.component";

@Component({
  selector: "app-header",
  standalone: true,
  imports: [CommonModule, ThemeToggleComponent],
  templateUrl: "./header.component.html",
})
export class HeaderComponent {
  title = input.required<string>();
  subtitle = input.required<string>();
  isDark = input.required<boolean>();
  toggleTheme = output<void>();
}

/* sys lib */
import { Component, output } from "@angular/core";
import { CommonModule } from "@angular/common";

@Component({
  selector: "app-swap-button",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./swap-button.component.html",
})
export class SwapButtonComponent {
  swap = output<void>();
}

/* sys lib */
import { Component, output } from "@angular/core";

/* components */
import { AppIconComponent } from "@components/icons/app-icon.component";

@Component({
  selector: "app-swap-button",
  standalone: true,
  imports: [AppIconComponent],
  templateUrl: "./swap-button.component.html",
})
export class SwapButtonComponent {
  swap = output<void>();
}

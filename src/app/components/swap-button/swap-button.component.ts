/* sys lib */
import { Component, input, output } from "@angular/core";

/* components */
import { AppIconComponent } from "@components/icons/icons.component";

@Component({
  selector: "app-swap-button",
  standalone: true,
  imports: [AppIconComponent],
  templateUrl: "./swap-button.component.html",
})
export class SwapButtonComponent {
  swap = output<void>();
}

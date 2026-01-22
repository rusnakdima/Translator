/* sys lib */
import { Component, input } from "@angular/core";
import { CommonModule } from "@angular/common";

@Component({
  selector: "app-toast",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./toast.component.html",
})
export class ToastComponent {
  message = input.required<string>();
  isVisible = input.required<boolean>();
}

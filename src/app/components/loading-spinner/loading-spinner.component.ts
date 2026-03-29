/* sys lib */
import { Component, input } from "@angular/core";

/* components */
import { AppIconComponent } from "@components/icons/app-icon.component";

@Component({
  selector: "app-loading-spinner",
  standalone: true,
  imports: [AppIconComponent],
  templateUrl: "./loading-spinner.component.html",
})
export class LoadingSpinnerComponent {
  isLoading = input.required<boolean>();
}

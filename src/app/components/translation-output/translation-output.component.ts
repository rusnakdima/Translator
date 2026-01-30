/* sys lib */
import { Component, input, output } from "@angular/core";
import { FormsModule } from "@angular/forms";

@Component({
  selector: "app-translation-output",
  standalone: true,
  imports: [FormsModule],
  templateUrl: "./translation-output.component.html",
})
export class TranslationOutputComponent {
  outputId = input.required<string>();
  placeholder = input.required<string>();
  translatedText = input.required<string>();
  copyClick = output<void>();
}

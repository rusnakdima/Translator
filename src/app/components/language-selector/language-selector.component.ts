/* sys lib */
import { Component, input, output } from "@angular/core";
import { CommonModule } from "@angular/common";
import { FormsModule } from "@angular/forms";

/* models */
import { Language } from "@models/translation.model";

@Component({
  selector: "app-language-selector",
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: "./language-selector.component.html",
})
export class LanguageSelectorComponent {
  labelId = input.required<string>();
  languages = input.required<Language[]>();
  selectedLang = input.required<string>();
  selectedLangChange = output<string>();
}

/* sys lib */
import { Component, input, output, ViewChild, ElementRef } from "@angular/core";
import { FormsModule } from "@angular/forms";

/* entities */
import { Language } from "@features/translation/entities/translation.entity";

/* components */
import { AppIconComponent } from "@components/icons/icons.component";

@Component({
  selector: "app-language-selector",
  standalone: true,
  imports: [FormsModule, AppIconComponent],
  templateUrl: "./language-selector.component.html",
})
export class LanguageSelectorComponent {
  @ViewChild("selectEl") selectRef!: ElementRef<HTMLSelectElement>;

  labelId = input.required<string>();
  languages = input.required<Language[]>();
  selectedLang = input.required<string>();
  selectedLangChange = output<string>();

  focus(): void {
    this.selectRef?.nativeElement.focus();
  }
}

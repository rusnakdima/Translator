/* sys lib */
import { Component, signal } from "@angular/core";

/* components */
import { HeaderComponent } from "@components/header/header.component";
import { TextInputComponent } from "@components/text-input/text-input.component";
import { LanguageSelectorComponent } from "@components/language-selector/language-selector.component";
import { SwapButtonComponent } from "@components/swap-button/swap-button.component";
import { TranslationOutputComponent } from "@components/translation-output/translation-output.component";
import { LoadingSpinnerComponent } from "@components/loading-spinner/loading-spinner.component";
import { ShortcutsOverlayComponent } from "@components/shortcuts-overlay/shortcuts-overlay.component";

/* models */
import { Language } from "@features/translation/models/translation.model";

@Component({
  selector: "app-translation",
  standalone: true,
  imports: [
    HeaderComponent,
    TextInputComponent,
    LanguageSelectorComponent,
    SwapButtonComponent,
    TranslationOutputComponent,
    LoadingSpinnerComponent,
    ShortcutsOverlayComponent,
  ],
  templateUrl: "./translation.component.html",
})
export class TranslationComponent {
  isDark = signal<boolean>(false);
  isLoading = signal<boolean>(false);
  showShortcuts = signal<boolean>(false);

  sourceText = signal<string>("");
  translatedText = signal<string>("");

  languages: Language[] = [
    { code: "en", name: "English", nativeName: "English" },
    { code: "es", name: "Spanish", nativeName: "Español" },
    { code: "fr", name: "French", nativeName: "Français" },
    { code: "de", name: "German", nativeName: "Deutsch" },
    { code: "it", name: "Italian", nativeName: "Italiano" },
    { code: "pt", name: "Portuguese", nativeName: "Português" },
    { code: "ru", name: "Russian", nativeName: "Русский" },
    { code: "zh", name: "Chinese", nativeName: "中文" },
    { code: "ja", name: "Japanese", nativeName: "日本語" },
    { code: "ko", name: "Korean", nativeName: "한국어" },
  ];

  sourceLang = signal<string>("en");
  targetLang = signal<string>("es");

  onToggleTheme(): void {
    this.isDark.update((v) => !v);
    document.documentElement.classList.toggle("dark");
  }

  onShowShortcuts(): void {
    this.showShortcuts.set(true);
  }

  onCloseShortcuts(): void {
    this.showShortcuts.set(false);
  }

  onSwap(): void {
    const temp = this.sourceLang();
    this.sourceLang.set(this.targetLang());
    this.targetLang.set(temp);
    const tempText = this.sourceText();
    this.sourceText.set(this.translatedText());
    this.translatedText.set(tempText);
  }

  onSourceTextChange(text: string): void {
    this.sourceText.set(text);
  }

  onTargetTextChange(text: string): void {
    this.translatedText.set(text);
  }

  onSourceLangChange(lang: string): void {
    this.sourceLang.set(lang);
  }

  onTargetLangChange(lang: string): void {
    this.targetLang.set(lang);
  }

  onCopy(): void {
    navigator.clipboard.writeText(this.translatedText());
  }
}

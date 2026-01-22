/* sys lib */
import { Component, OnInit, inject, signal, effect } from "@angular/core";
import { CommonModule } from "@angular/common";
import { FormsModule } from "@angular/forms";
import { listen } from "@tauri-apps/api/event";

/* services */
import { TranslationService } from "@services/translation.service";

/* models */
import { Language, TranslationResponse } from "@models/translation.model";

/* helpers */
import { ToastHelper } from "@helpers/toast.helper";

/* components */
import { HeaderComponent } from "@components/header/header.component";
import { LanguageSelectorComponent } from "@components/language-selector/language-selector.component";
import { TextInputComponent } from "@components/text-input/text-input.component";
import { TranslationOutputComponent } from "@components/translation-output/translation-output.component";
import { SwapButtonComponent } from "@components/swap-button/swap-button.component";
import { LoadingSpinnerComponent } from "@components/loading-spinner/loading-spinner.component";

interface TranslationResultEvent {
  requestId: number;
  text: string;
  sourceLang: string;
  targetLang: string;
  response: {
    status: string;
    message: string;
    data: TranslationResponse;
  };
}

@Component({
  selector: "app-translation",
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    HeaderComponent,
    LanguageSelectorComponent,
    TextInputComponent,
    TranslationOutputComponent,
    SwapButtonComponent,
    LoadingSpinnerComponent,
  ],
  templateUrl: "./translation.component.html",
})
export class TranslationComponent implements OnInit {
  private translationService = inject(TranslationService);

  readonly languages = signal<Language[]>([]);
  readonly translatedText = signal<string>("");
  readonly charCount = signal<string>("0/5000");
  readonly isLoading = signal(false);
  readonly error = signal<string | null>(null);
  readonly isDark = signal<boolean>(true);

  sourceLang = signal<string>("");
  targetLang = signal<string>("es");
  inputText = signal<string>("");

  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private currentRequestId: number | null = null;

  onInputTextChange(value: string): void {
    this.inputText.set(value);
    this.onInputChange();
  }

  constructor() {
    effect(() => {
      const count = this.inputText().length;
      const maxChars = this.translationService.getMaxChars();
      this.charCount.set(`${count}/${maxChars}`);
    });

    effect(() => {
      const isDark = this.isDark();
      const element = document.documentElement;
      if (isDark) {
        element.classList.remove("light");
        element.classList.add("dark");
      } else {
        element.classList.remove("dark");
        element.classList.add("light");
      }
    });
  }

  async ngOnInit(): Promise<void> {
    ToastHelper.initialize();
    await this.loadLanguages();
    await this.setupEventListener();
  }

  private async setupEventListener(): Promise<void> {
    await listen<TranslationResultEvent>("translation-result", (event) => {
      const { requestId, response } = event.payload;

      if (
        this.currentRequestId === null ||
        requestId !== this.currentRequestId
      ) {
        return;
      }

      this.isLoading.set(false);
      this.currentRequestId = null;

      if (response.status === "error") {
        this.error.set(response.message);
      } else {
        this.error.set(null);
        if (response.data && response.data.translatedText) {
          this.translatedText.set(response.data.translatedText);
        }
      }
    });
  }

  async loadLanguages(): Promise<void> {
    const languages = await this.translationService.getSupportedLanguages();
    this.languages.set(languages);

    if (languages.length > 0) {
      this.sourceLang.set(languages[0].code);
    }
  }

  toggleTheme(): void {
    this.isDark.update((v) => !v);
  }

  async triggerTranslation(): Promise<void> {
    const text = this.inputText().trim();

    if (!text) {
      this.translatedText.set("");
      this.error.set(null);
      return;
    }

    const { invoke } = await import("@tauri-apps/api/core");

    const result = await invoke<number>("translate_text", {
      text,
      sourceLang: this.sourceLang(),
      targetLang: this.targetLang(),
    });

    this.currentRequestId = result;
    this.isLoading.set(true);
    this.error.set(null);
  }

  async translate(): Promise<void> {
    await this.triggerTranslation();
  }

  onInputChange(): void {
    this.cancelPending();

    const text = this.inputText().trim();
    if (!text) {
      this.translatedText.set("");
      this.error.set(null);
      return;
    }

    this.debounceTimer = setTimeout(() => {
      this.triggerTranslation();
    }, 500);
  }

  onSourceLangChange(lang: string): void {
    this.cancelPending();
    this.sourceLang.set(lang);

    const text = this.inputText().trim();
    if (!text) {
      return;
    }

    this.debounceTimer = setTimeout(() => {
      this.triggerTranslation();
    }, 500);
  }

  onTargetLangChange(lang: string): void {
    this.cancelPending();
    this.targetLang.set(lang);

    const text = this.inputText().trim();
    if (!text) {
      return;
    }

    this.debounceTimer = setTimeout(() => {
      this.triggerTranslation();
    }, 500);
  }

  private cancelPending(): void {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
  }

  swapLanguages(): void {
    const temp = this.sourceLang();
    this.sourceLang.set(this.targetLang());
    this.targetLang.set(temp);

    const text = this.inputText().trim();
    if (text) {
      this.cancelPending();
      this.debounceTimer = setTimeout(() => {
        this.triggerTranslation();
      }, 500);
    }
  }

  async copyToClipboard(): Promise<void> {
    const text = this.translatedText();
    if (!text) {
      ToastHelper.show("Nothing to copy");
      return Promise.resolve();
    }

    return navigator.clipboard
      .writeText(text)
      .then(() => ToastHelper.show("Copied to clipboard!"))
      .catch(() => ToastHelper.show("Failed to copy"));
  }

  handleKeyDown(event: KeyboardEvent): void {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      this.cancelPending();
      this.triggerTranslation();
    }
  }
}

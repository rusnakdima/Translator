/* sys lib */
import { Component, OnInit, inject, signal, effect } from "@angular/core";
import { FormsModule } from "@angular/forms";
import { listen } from "@tauri-apps/api/event";

/* services */
import { TranslationService } from "@services/translation.service";

/* models */
import { Language, TranslationResponse } from "@models/translation.model";

/* helpers */
import { ToastHelper } from "@helpers/toast.helper";

/* constants */
import {
  RESPONSE_STATUS,
  TAURI_EVENTS,
  ToastKind,
} from "@constants/app.constants";

/* components */
import { HeaderComponent } from "@components/header/header.component";
import { LanguageSelectorComponent } from "@components/language-selector/language-selector.component";
import { TextInputComponent } from "@components/text-input/text-input.component";
import { TranslationOutputComponent } from "@components/translation-output/translation-output.component";
import { SwapButtonComponent } from "@components/swap-button/swap-button.component";
import { LoadingSpinnerComponent } from "@components/loading-spinner/loading-spinner.component";
import { AppIconComponent } from "@components/icons/app-icon.component";

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
    FormsModule,
    HeaderComponent,
    LanguageSelectorComponent,
    TextInputComponent,
    TranslationOutputComponent,
    SwapButtonComponent,
    LoadingSpinnerComponent,
    AppIconComponent,
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

  onClearInput(): void {
    this.inputText.set("");
    this.translatedText.set("");
    this.error.set(null);
    this.cancelPending();
    ToastHelper.show("Text cleared", ToastKind.Info);
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
    await this.loadLanguages();
    await this.setupEventListener();
  }

  private async setupEventListener(): Promise<void> {
    await listen<TranslationResultEvent>(
      TAURI_EVENTS.translationResult,
      (event) => {
        const { requestId, response } = event.payload;

        if (
          this.currentRequestId === null ||
          requestId !== this.currentRequestId
        ) {
          return;
        }

        this.isLoading.set(false);
        this.currentRequestId = null;

        if (response.status === RESPONSE_STATUS.error) {
          this.error.set(response.message);
          ToastHelper.show(response.message, ToastKind.Error);
        } else {
          this.error.set(null);
          if (response.data?.translatedText) {
            this.translatedText.set(response.data.translatedText);
            ToastHelper.show("Text translated", ToastKind.Success);
          }
        }
      },
    );
  }

  async loadLanguages(): Promise<void> {
    try {
      const languages = await this.translationService.getSupportedLanguages();
      this.languages.set(languages);

      if (languages.length > 0) {
        this.sourceLang.set(languages[0].code);
      }
    } catch {
      this.languages.set([]);
      ToastHelper.show("Failed to load languages", ToastKind.Error);
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
    this.sourceLang.set(lang);
    this.scheduleTranslationAfterLangChange();
  }

  onTargetLangChange(lang: string): void {
    this.targetLang.set(lang);
    this.scheduleTranslationAfterLangChange();
  }

  private scheduleTranslationAfterLangChange(): void {
    this.cancelPending();

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
    const tempLang = this.sourceLang();
    this.sourceLang.set(this.targetLang());
    this.targetLang.set(tempLang);

    const tempText = this.inputText();
    this.inputText.set(this.translatedText());
    this.translatedText.set(tempText);

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
      ToastHelper.show("Nothing to copy", ToastKind.Info);
      return;
    }

    try {
      await navigator.clipboard.writeText(text);
      ToastHelper.show("Copied to clipboard!", ToastKind.Success);
    } catch {
      ToastHelper.show("Failed to copy", ToastKind.Error);
    }
  }

  handleKeyDown(event: KeyboardEvent): void {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      this.cancelPending();
      this.triggerTranslation();
    }
  }
}

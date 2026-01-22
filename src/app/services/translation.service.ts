/* sys lib */
import { Injectable, signal } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";

/* models */
import {
  Language,
  LanguagesResponse,
  TranslationRequest,
  TranslationResponse,
} from "@models/translation.model";
import { Response } from "@models/response.model";

@Injectable({
  providedIn: "root",
})
export class TranslationService {
  private readonly maxChars = 5000;

  readonly isLoading = signal(false);
  readonly error = signal<string | null>(null);

  async getSupportedLanguages(): Promise<Language[]> {
    try {
      const response = await invoke<Response<LanguagesResponse>>(
        "get_supported_languages",
      );
      if (response.status === "error") {
        throw new Error(response.message);
      }
      return response.data.languages;
    } catch (err) {
      this.error.set("Failed to load languages");
      console.error("Error loading languages:", err);
      return [];
    }
  }

  async translate(
    request: TranslationRequest,
    _signal?: AbortSignal,
  ): Promise<TranslationResponse> {
    if (!request.text.trim()) {
      throw new Error("Please enter some text to translate");
    }

    if (request.text.length > this.maxChars) {
      throw new Error("Text is too long. Maximum 5000 characters.");
    }

    this.isLoading.set(true);
    this.error.set(null);

    try {
      const response = await invoke<Response<TranslationResponse>>(
        "translate_text",
        {
          text: request.text,
          sourceLang: request.sourceLang,
          targetLang: request.targetLang,
        },
      );
      if (response.status === "error") {
        throw new Error(response.message);
      }
      return response.data;
    } catch (err) {
      const message = err instanceof Error ? err.message : "Translation failed";
      this.error.set(message);
      throw new Error(message);
    } finally {
      this.isLoading.set(false);
    }
  }

  getMaxChars(): number {
    return this.maxChars;
  }
}

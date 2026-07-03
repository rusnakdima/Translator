/* sys lib */
import { Injectable, inject } from "@angular/core";

/* shared */
import { InvokeWrapperService, type Response } from "@tauri-front/shared";

/* entities */
import {
  Language,
  LanguagesResponse,
} from "@features/translation/entities/translation.entity";

@Injectable({
  providedIn: "root",
})
export class TranslationService {
  private readonly invokeWrapper = inject(InvokeWrapperService);
  private readonly maxChars = 5000;

  async getSupportedLanguages(): Promise<Language[]> {
    const response = await this.invokeWrapper.invoke<
      Response<LanguagesResponse>
    >("get_supported_languages");
    if (!response.data) {
      throw new Error(response.message || "Failed to load languages");
    }
    return response.data.languages ?? [];
  }

  getMaxChars(): number {
    return this.maxChars;
  }
}

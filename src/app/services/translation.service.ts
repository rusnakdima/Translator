/* sys lib */
import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";

/* models */
import { Language, LanguagesResponse } from "@models/translation.model";
import { Response } from "@models/response.model";
import { RESPONSE_STATUS } from "@constants/app.constants";

@Injectable({
  providedIn: "root",
})
export class TranslationService {
  private readonly maxChars = 5000;

  async getSupportedLanguages(): Promise<Language[]> {
    const response = await invoke<Response<LanguagesResponse>>(
      "get_supported_languages",
    );
    if (response.status === RESPONSE_STATUS.error) {
      throw new Error(response.message);
    }
    return response.data.languages;
  }

  getMaxChars(): number {
    return this.maxChars;
  }
}

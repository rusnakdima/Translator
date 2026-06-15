/* sys lib */
import { Injectable, inject } from "@angular/core";

/* services */
import { TauriApiService } from "@api/tauri-api.service";

/* models */
import {
  Language,
  LanguagesResponse,
} from "@features/translation/models/translation.model";
import { Response } from "@features/translation/models/response.model";
import { RESPONSE_STATUS } from "@shared/utils/constants";

@Injectable({
  providedIn: "root",
})
export class TranslationService {
  private readonly tauriApi = inject(TauriApiService);
  private readonly maxChars = 5000;

  async getSupportedLanguages(): Promise<Language[]> {
    const response = await this.tauriApi.invoke<Response<LanguagesResponse>>(
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

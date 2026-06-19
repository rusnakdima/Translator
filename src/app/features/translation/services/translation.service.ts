/* sys lib */
import { Injectable, inject } from "@angular/core";

/* services */
import { InvokeWrapperService } from "@app/services/invoke-wrapper.service";

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
  private readonly invokeWrapper = inject(InvokeWrapperService);
  private readonly maxChars = 5000;

  async getSupportedLanguages(): Promise<Language[]> {
    const response = await this.invokeWrapper.invoke<
      Response<LanguagesResponse>
    >("get_supported_languages");
    if (response.status === RESPONSE_STATUS.error) {
      throw new Error(response.message);
    }
    return response.data.languages;
  }

  getMaxChars(): number {
    return this.maxChars;
  }
}

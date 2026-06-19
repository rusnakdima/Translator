/* sys lib */
import { Injectable, inject } from "@angular/core";

/* services */
import { InvokeWrapperService } from "@app/services/services.invoke-wrapper.service";

/* entities */
import {
  Language,
  LanguagesResponse,
} from "@features/translation/entities/translation.entity";
import { Response } from "@features/translation/entities/response.entity";
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

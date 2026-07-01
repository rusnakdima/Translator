/* sys lib */
import { Injectable } from "@angular/core";

/* shared */
import { InvokeWrapperService } from "@tauri-front/shared";

/* entities */
import {
  Language,
  LanguagesResponse,
} from "@features/translation/entities/translation.entity";

interface ApiResponse<T> {
  status: string;
  message: string;
  data: T | null;
}

@Injectable({
  providedIn: "root",
})
export class TranslationService {
  private readonly invokeWrapper = new InvokeWrapperService();
  private readonly maxChars = 5000;

  async getSupportedLanguages(): Promise<Language[]> {
    const response = await this.invokeWrapper.invoke<
      ApiResponse<LanguagesResponse>
    >("get_supported_languages");
    if (response.status === "error" || response.status === "Error") {
      throw new Error(response.message);
    }
    return response.data?.languages ?? [];
  }

  getMaxChars(): number {
    return this.maxChars;
  }
}

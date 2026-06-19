import { Injectable, inject } from "@angular/core";
import { TauriApiService } from "@api/api.api.service";

export interface InvokeOptions {
  timeout?: number;
  retryCount?: number;
  retryDelay?: number;
}

@Injectable({
  providedIn: "root",
})
export class InvokeWrapperService {
  private readonly api = inject(TauriApiService);

  async invoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
    options: InvokeOptions = {},
  ): Promise<T> {
    const { retryCount = 0, retryDelay = 1000 } = options;

    let lastError: unknown;

    for (let attempt = 0; attempt <= retryCount; attempt++) {
      try {
        return await this.api.invoke<T>(cmd, args);
      } catch (error) {
        lastError = error;

        if (attempt < retryCount) {
          await new Promise((resolve) => setTimeout(resolve, retryDelay));
        }
      }
    }

    throw lastError;
  }
}

import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { getLoggingService } from "@tauri-apps/logger";

@Injectable({ providedIn: "root" })
export class TauriApiService {
  private readonly loggingService = getLoggingService();
  private readonly defaultTimeout = 30000;

  async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const operationId = this.loggingService.logServiceEntry(
      "TauriApiService",
      "invoke",
      { cmd, args },
    );

    try {
      const result = await this.withTimeout<T>(
        invoke<T>(cmd, args),
        this.defaultTimeout,
        `Tauri command '${cmd}' timed out`,
      );

      this.loggingService.logServiceExit(
        "TauriApiService",
        "invoke",
        operationId,
        0,
        { cmd, success: true },
      );
      return result;
    } catch (error) {
      this.loggingService.error(`Tauri command '${cmd}' failed`, error, {
        cmd,
        args,
      });
      this.loggingService.logServiceExit(
        "TauriApiService",
        "invoke",
        operationId,
        0,
        { cmd, success: false },
      );
      throw error;
    }
  }

  private async withTimeout<T>(
    promise: Promise<T>,
    ms: number,
    timeoutMessage: string,
  ): Promise<T> {
    let timeoutId: ReturnType<typeof setTimeout> | undefined;

    const timeoutPromise = new Promise<never>((_, reject) => {
      timeoutId = setTimeout(() => reject(new Error(timeoutMessage)), ms);
    });

    try {
      return await Promise.race([promise, timeoutPromise]);
    } finally {
      if (timeoutId !== undefined) {
        clearTimeout(timeoutId);
      }
    }
  }
}

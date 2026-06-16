import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";
import { loggerService } from "../core/services/logger.service";

@Injectable({ providedIn: "root" })
export class TauriApiService {
  private readonly defaultTimeout = 30000;

  async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    loggerService.debug(`Invoking Tauri command: ${cmd}`, "TauriApiService", {
      cmd,
      args,
    });

    try {
      const result = await this.withTimeout<T>(
        invoke<T>(cmd, args),
        this.defaultTimeout,
        `Tauri command '${cmd}' timed out`,
      );

      loggerService.info(`Tauri command succeeded: ${cmd}`, "TauriApiService", {
        cmd,
        success: true,
      });
      return result;
    } catch (error) {
      loggerService.error(`Tauri command '${cmd}' failed`, "TauriApiService", {
        cmd,
        args,
        error,
      });
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

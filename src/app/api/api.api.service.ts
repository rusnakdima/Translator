import { Injectable } from "@angular/core";
import { invoke } from "@tauri-apps/api/core";

@Injectable({ providedIn: "root" })
export class TauriApiService {
  private readonly defaultTimeout = 30000;

  async invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await this.withTimeout<T>(
        invoke<T>(cmd, args),
        this.defaultTimeout,
        `Tauri command '${cmd}' timed out`,
      );
    } catch (error) {
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

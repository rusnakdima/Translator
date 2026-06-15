import { getLoggingService } from "@tauri-apps/logger";

export interface ErrorDetails {
  message: string;
  code?: string;
  originalError?: unknown;
}

export class ErrorHandlerService {
  private static loggingService = getLoggingService();

  static handle(error: unknown, context?: string): Error {
    const message = error instanceof Error ? error.message : String(error);
    const details: ErrorDetails = {
      message,
      originalError: error,
    };

    if (context) {
      this.loggingService.error(`[${context}]`, details);
    } else {
      this.loggingService.error("Unhandled error", details);
    }

    return new Error(message);
  }

  static getMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }
}

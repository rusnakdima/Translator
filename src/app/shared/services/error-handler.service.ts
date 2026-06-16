import { loggerService } from "../../core/services/logger.service";

export interface ErrorDetails {
  message: string;
  code?: string;
  originalError?: unknown;
}

export class ErrorHandlerService {
  static handle(error: unknown, context?: string): Error {
    const message = error instanceof Error ? error.message : String(error);
    const details: ErrorDetails = {
      message,
      originalError: error,
    };

    if (context) {
      loggerService.error(`[${context}]`, "ErrorHandlerService", details);
    } else {
      loggerService.error("Unhandled error", "ErrorHandlerService", details);
    }

    return new Error(message);
  }

  static getMessage(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
  }
}

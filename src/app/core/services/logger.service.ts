import { inject } from "@angular/core";
import { InvokeWrapperService } from "@app/services/invoke-wrapper.service";

export enum LogLevel {
  Debug = 0,
  Info = 1,
  Warn = 2,
  Error = 3,
}

export interface LogEntry {
  timestamp: Date;
  level: LogLevel;
  message: string;
  context?: string;
  data?: unknown;
}

export class LoggerService {
  private static instance: LoggerService;
  private logs: LogEntry[] = [];
  private maxEntries = 1000;
  private currentLevel: LogLevel = LogLevel.Info;
  private invokeWrapper = inject(InvokeWrapperService);

  private constructor() {}

  static getInstance(): LoggerService {
    if (!LoggerService.instance) {
      LoggerService.instance = new LoggerService();
    }
    return LoggerService.instance;
  }

  debug(message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.Debug, message, context, data);
  }

  info(message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.Info, message, context, data);
  }

  warn(message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.Warn, message, context, data);
  }

  error(message: string, context?: string, data?: unknown): void {
    this.log(LogLevel.Error, message, context, data);
  }

  private logToBackend(
    level: LogLevel,
    message: string,
    context?: string,
  ): void {
    const levelMap: Record<LogLevel, string> = {
      [LogLevel.Debug]: "debug",
      [LogLevel.Info]: "info",
      [LogLevel.Warn]: "warn",
      [LogLevel.Error]: "error",
    };
    this.invokeWrapper
      .invoke("log_message", {
        level: levelMap[level],
        component: context || "app",
        message,
      })
      .catch(() => {});
  }

  private log(
    level: LogLevel,
    message: string,
    context?: string,
    data?: unknown,
  ): void {
    if (level < this.currentLevel) {
      return;
    }

    const entry: LogEntry = {
      timestamp: new Date(),
      level,
      message,
      context,
      data,
    };

    this.logs.push(entry);

    if (this.logs.length > this.maxEntries) {
      this.logs.shift();
    }

    const levelName = LogLevel[level];
    const timestamp = entry.timestamp.toISOString();
    console.log(
      `[${timestamp}] ${levelName} - ${context || "app"}: ${message}`,
      data || "",
    );

    this.logToBackend(level, message, context);
  }

  getLogs(): LogEntry[] {
    return [...this.logs];
  }

  clearLogs(): void {
    this.logs = [];
  }

  setLevel(level: LogLevel): void {
    this.currentLevel = level;
  }

  getLevel(): LogLevel {
    return this.currentLevel;
  }
}

export const loggerService = LoggerService.getInstance();

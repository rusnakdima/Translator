import { Injectable, signal } from "@angular/core";

export interface Language {
  code: string;
  name: string;
}

@Injectable({ providedIn: "root" })
export class GlobalStateService {
  private static _instance: GlobalStateService;

  sourceLang = signal<string>("en");
  targetLang = signal<string>("ru");
  appLocale = signal<"en" | "ru">("en");

  constructor() {
    GlobalStateService._instance = this;
  }

  static get instance(): GlobalStateService {
    return GlobalStateService._instance;
  }

  setSourceLang(code: string): void {
    this.sourceLang.set(code);
  }

  getSourceLang(): string {
    return this.sourceLang();
  }

  setTargetLang(code: string): void {
    this.targetLang.set(code);
  }

  getTargetLang(): string {
    return this.targetLang();
  }

  swapLanguages(): void {
    const tmp = this.sourceLang();
    this.sourceLang.set(this.targetLang());
    this.targetLang.set(tmp);
  }

  setAppLocale(locale: "en" | "ru"): void {
    this.appLocale.set(locale);
  }

  getAppLocale(): "en" | "ru" {
    return this.appLocale();
  }
}

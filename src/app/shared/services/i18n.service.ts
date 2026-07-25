import { Injectable } from "@angular/core";
import { I18nService as SharedI18nService } from "@tauri-front/shared";

@Injectable({ providedIn: "root" })
export class I18nService {
  private static _instance: I18nService;
  private _service: SharedI18nService;

  constructor(shared: SharedI18nService) {
    this._service = shared;
    I18nService._instance = this;
  }

  static get instance(): I18nService {
    return I18nService._instance;
  }

  setLocale(locale: "en" | "ru"): void {
    this._service.setLocale(locale);
  }

  get locale() {
    return this._service.locale;
  }

  get translations(): Record<string, string> {
    return this._service.translations;
  }

  t(key: string): string {
    return this._service.t(key);
  }

  getAvailableLocales(): Array<"en" | "ru"> {
    return this._service.getAvailableLocales();
  }
}

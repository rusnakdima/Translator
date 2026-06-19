export interface Language {
  code: string;
  name: string;
}

export interface LanguagesResponse {
  languages: Language[];
}

export interface TranslationResponse {
  translatedText: string;
  sourceLang: string;
  targetLang: string;
}

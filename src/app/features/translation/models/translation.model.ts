export interface Language {
  code: string;
  name: string;
}

export interface LanguagesResponse {
  languages: Language[];
}

export interface TranslationRequest {
  text: string;
  sourceLang: string;
  targetLang: string;
}

export interface TranslationResponse {
  translatedText: string;
  sourceLang: string;
  targetLang: string;
}

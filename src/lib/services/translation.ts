import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { TAURI_EVENTS } from "$lib/utils/constants";

export interface Language {
  code: string;
  name: string;
}

export interface LanguagesResponse {
  languages: Language[];
}

export interface Response<T> {
  status: string;
  message: string;
  data: T;
}

export interface TranslationResultPayload {
  requestId: number;
  text: string;
  sourceLang: string;
  targetLang: string;
  response: Response<{ translatedText: string }>;
}

const maxChars = 5000;

export async function getSupportedLanguages(): Promise<Language[]> {
  const response = await invoke<Response<LanguagesResponse>>(
    "get_supported_languages",
  );
  if (!response.data) {
    throw new Error(response.message || "Failed to load languages");
  }
  return response.data.languages ?? [];
}

export function getMaxChars(): number {
  return maxChars;
}

export async function translateText(
  text: string,
  sourceLang: string,
  targetLang: string,
): Promise<number> {
  return await invoke<number>("translate_text", {
    text,
    sourceLang,
    targetLang,
  });
}

export async function listenForTranslationResult(
  callback: (payload: TranslationResultPayload) => void,
): Promise<UnlistenFn> {
  return await listen<TranslationResultPayload>(
    TAURI_EVENTS.translationResult,
    (event) => {
      callback(event.payload);
    },
  );
}

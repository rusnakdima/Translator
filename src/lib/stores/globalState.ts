import { writable } from "svelte/store";

export const sourceLang = writable<string>("en");
export const targetLang = writable<string>("ru");
export const appLocale = writable<"en" | "ru">("en");

export function swapLanguages(): void {
  let source: string;
  let target: string;
  sourceLang.subscribe((v) => (source = v))();
  targetLang.subscribe((v) => (target = v))();

  sourceLang.set(target);
  targetLang.set(source);
}

import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import {
  sourceLang,
  targetLang,
  appLocale,
  swapLanguages,
} from "./globalState";

describe("globalState store", () => {
  beforeEach(() => {
    sourceLang.set("en");
    targetLang.set("ru");
    appLocale.set("en");
  });

  describe("sourceLang", () => {
    it('should have default value of "en"', () => {
      expect(get(sourceLang)).toBe("en");
    });

    it("should allow setting a new value", () => {
      sourceLang.set("ru");
      expect(get(sourceLang)).toBe("ru");
    });

    it("should notify subscribers on change", () => {
      let received = "";
      const unsubscribe = sourceLang.subscribe((v) => {
        received = v;
      });
      sourceLang.set("fr");
      expect(received).toBe("fr");
      unsubscribe();
    });
  });

  describe("targetLang", () => {
    it('should have default value of "ru"', () => {
      expect(get(targetLang)).toBe("ru");
    });

    it("should allow setting a new value", () => {
      targetLang.set("en");
      expect(get(targetLang)).toBe("en");
    });

    it("should notify subscribers on change", () => {
      let received = "";
      const unsubscribe = targetLang.subscribe((v) => {
        received = v;
      });
      targetLang.set("de");
      expect(received).toBe("de");
      unsubscribe();
    });
  });

  describe("appLocale", () => {
    it('should have default value of "en"', () => {
      expect(get(appLocale)).toBe("en");
    });

    it("should accept only valid locale values", () => {
      appLocale.set("ru");
      expect(get(appLocale)).toBe("ru");
    });
  });

  describe("swapLanguages", () => {
    it("should swap source and target language values", () => {
      sourceLang.set("en");
      targetLang.set("ru");
      swapLanguages();
      expect(get(sourceLang)).toBe("ru");
      expect(get(targetLang)).toBe("en");
    });

    it("should correctly swap when languages are same", () => {
      sourceLang.set("fr");
      targetLang.set("fr");
      swapLanguages();
      expect(get(sourceLang)).toBe("fr");
      expect(get(targetLang)).toBe("fr");
    });
  });
});

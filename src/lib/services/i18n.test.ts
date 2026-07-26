import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { locale, setLocale, getAvailableLocales, t, currentTranslations } from './i18n';

describe('i18n service', () => {
  beforeEach(() => {
    locale.set('en');
  });

  describe('locale store', () => {
    it('should have default value of "en"', () => {
      expect(get(locale)).toBe('en');
    });

    it('should allow setting to "ru"', () => {
      setLocale('ru');
      expect(get(locale)).toBe('ru');
    });

    it('should accept only valid Locale values', () => {
      setLocale('ru');
      expect(get(locale)).toBe('ru');
      setLocale('en');
      expect(get(locale)).toBe('en');
    });

    it('should notify subscribers on change', () => {
      let received = 'en';
      const unsubscribe = locale.subscribe((v) => { received = v; });
      setLocale('ru');
      expect(received).toBe('ru');
      unsubscribe();
    });
  });

  describe('getAvailableLocales', () => {
    it('should return both en and ru', () => {
      const locales = getAvailableLocales();
      expect(locales).toContain('en');
      expect(locales).toContain('ru');
    });

    it('should return exactly 2 locales', () => {
      expect(getAvailableLocales()).toHaveLength(2);
    });
  });

  describe('currentTranslations derived store', () => {
    it('should return English translations when locale is en', () => {
      locale.set('en');
      const dict = get(currentTranslations);
      expect(dict['app.title']).toBe('Translator');
      expect(dict['translation.source']).toBe('Source language');
    });

    it('should return Russian translations when locale is ru', () => {
      locale.set('ru');
      const dict = get(currentTranslations);
      expect(dict['app.title']).toBe('Переводчик');
      expect(dict['translation.source']).toBe('Исходный язык');
    });
  });

  describe('t() translation function', () => {
    it('should return translation for known key in English', () => {
      locale.set('en');
      expect(t('app.title')).toBe('Translator');
      expect(t('translation.translate')).toBe('Translate');
    });

    it('should return translation for known key in Russian', () => {
      locale.set('ru');
      expect(t('app.title')).toBe('Переводчик');
      expect(t('translation.translate')).toBe('Перевести');
    });

    it('should return the key itself when translation is missing', () => {
      locale.set('en');
      expect(t('nonexistent.key')).toBe('nonexistent.key');
    });

    it('should replace parameters in translation string', () => {
      locale.set('en');
      const result = t('translation.charCount', { count: 42, max: 100 });
      expect(result).toBe('42 / 100 characters');
    });

    it('should handle numeric parameters', () => {
      locale.set('en');
      const result = t('translation.charCount', { count: 0, max: 5000 });
      expect(result).toBe('0 / 5000 characters');
    });

    it('should not modify string when no params provided for param string', () => {
      locale.set('en');
      expect(t('app.title')).toBe('Translator');
    });
  });

  describe('setLocale', () => {
    it('should update the locale store', () => {
      setLocale('ru');
      expect(get(locale)).toBe('ru');
      setLocale('en');
      expect(get(locale)).toBe('en');
    });
  });
});

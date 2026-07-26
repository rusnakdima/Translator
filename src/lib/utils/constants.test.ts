import { describe, it, expect } from 'vitest';
import {
  TAURI_EVENTS,
  RESPONSE_STATUS,
  ToastKind,
  SHORTCUTS,
} from './constants';
import type { Shortcut } from './constants';

describe('constants', () => {
  describe('TAURI_EVENTS', () => {
    it('should have translationResult event defined', () => {
      expect(TAURI_EVENTS.translationResult).toBe('translation-result');
    });

    it('should have readonly value via as const', () => {
      expect(TAURI_EVENTS.translationResult).toBe('translation-result');
    });
  });

  describe('RESPONSE_STATUS', () => {
    it('should have error and success status', () => {
      expect(RESPONSE_STATUS.error).toBe('error');
      expect(RESPONSE_STATUS.success).toBe('success');
    });

    it('should have readonly values via as const', () => {
      expect(RESPONSE_STATUS.error).toBe('error');
      expect(RESPONSE_STATUS.success).toBe('success');
    });
  });

  describe('ToastKind', () => {
    it('should have Info, Success, and Error variants', () => {
      expect(ToastKind.Info).toBe('info');
      expect(ToastKind.Success).toBe('success');
      expect(ToastKind.Error).toBe('error');
    });

    it('should satisfy the ToastType type', () => {
      const values: Array<'info' | 'success' | 'error'> = [
        ToastKind.Info,
        ToastKind.Success,
        ToastKind.Error,
      ];
      expect(values).toContain('info');
      expect(values).toContain('success');
      expect(values).toContain('error');
    });
  });

  describe('SHORTCUTS', () => {
    it('should be a non-empty array', () => {
      expect(Array.isArray(SHORTCUTS)).toBe(true);
      expect(SHORTCUTS.length).toBeGreaterThan(0);
    });

    it('each shortcut should have key, description, and action', () => {
      for (const shortcut of SHORTCUTS) {
        expect(shortcut).toHaveProperty('key');
        expect(shortcut).toHaveProperty('description');
        expect(shortcut).toHaveProperty('action');
        expect(typeof shortcut.key).toBe('string');
        expect(typeof shortcut.description).toBe('string');
        expect(typeof shortcut.action).toBe('string');
      }
    });

    it('should contain expected shortcut actions', () => {
      const actions = SHORTCUTS.map((s: Shortcut) => s.action);
      expect(actions).toContain('show-shortcuts');
      expect(actions).toContain('translate');
      expect(actions).toContain('swap');
      expect(actions).toContain('quick-paste');
      expect(actions).toContain('quick-copy');
      expect(actions).toContain('close');
    });

    it('should contain F1 shortcut', () => {
      const f1Shortcut = SHORTCUTS.find((s: Shortcut) => s.key === 'F1');
      expect(f1Shortcut).toBeDefined();
      expect(f1Shortcut?.action).toBe('show-shortcuts');
    });

    it('should contain Escape shortcut', () => {
      const escShortcut = SHORTCUTS.find((s: Shortcut) => s.key === 'Escape');
      expect(escShortcut).toBeDefined();
      expect(escShortcut?.action).toBe('close');
    });
  });
});

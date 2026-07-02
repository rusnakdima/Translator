/* sys lib */
import {
  Component,
  OnInit,
  OnDestroy,
  HostListener,
  CUSTOM_ELEMENTS_SCHEMA,
  inject,
  ElementRef,
  ViewChild,
} from "@angular/core";
import { CommonModule } from "@angular/common";

/* tauri */
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

/* shared */
import {
  SchemaRendererService,
  EventBusService,
  InvokeWrapperService,
  StyleThemeService,
  ShortcutService,
  I18nService,
  GlobalStateService,
} from "@tauri-front/shared";

/* features */
import { TranslationService } from "@features/translation/services/translation.service";

/* utils */
import { ToastHelper } from "@shared/utils/toast.helper";
import { ToastKind } from "@shared/utils/constants";

interface SchemaPayload {
  version: string;
  app: { id: string; name: string; theme?: string; style?: string };
  pages: Array<{
    id: string;
    route?: string;
    meta?: { title?: string };
    canvasElements: Array<{
      id: string;
      componentId: string;
      props?: Record<string, unknown>;
      classes?: string;
      children?: string[];
      events?: Record<string, Array<{ handler: string; params?: unknown }>>;
    }>;
  }>;
  layouts: unknown[];
  shortcuts?: Array<{
    id: string;
    key: string;
    modifiers?: string[];
    label: string;
    category?: string;
    handler: string;
    enabled?: boolean;
  }>;
}

@Component({
  selector: "app-root",
  standalone: true,
  imports: [CommonModule],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  template: `<div #schemaContainer></div>`,
})
export class App implements OnInit, OnDestroy {
  @ViewChild("schemaContainer") schemaContainer!: ElementRef<HTMLElement>;

  private schemaRenderer = inject(SchemaRendererService);
  private eventBus = inject(EventBusService);
  private translationService = inject(TranslationService);
  private themeService = inject(StyleThemeService);
  private shortcutService = inject(ShortcutService);

  languages: Array<{ code: string; name: string }> = [];
  sourceLang = "";
  targetLang = "es";
  inputText = "";
  translatedText = "";
  debounceTimer: ReturnType<typeof setTimeout> | null = null;
  currentRequestId: number | null = null;
  showShortcuts = false;

  private eventBusUnsubs: Array<() => void> = [];

  async ngOnInit() {
    await this.loadSchema();
    this.setupEventBridge();
    await this.loadLanguages();
    await this.setupTranslationListener();

    // Initialize theme from schema and localStorage
    const style = (this as any)._schema?.app?.style ?? "material-design-v3";
    this.themeService.loadTheme(style);
    const savedDark = localStorage.getItem("dark_mode");
    if (savedDark !== null) {
      this.themeService.setDarkMode(savedDark === "true");
    }

    // Subscribe to theme changes
    const themeSub = this.themeService.themeChanged$.subscribe(() => {
      this._applyThemeState();
    });
    this.eventBusUnsubs.push(() => themeSub.unsubscribe());
  }

  ngOnDestroy() {
    this.eventBusUnsubs.forEach((unsub) => unsub());
    this.cancelPending();
  }

  private async loadSchema() {
    try {
      const result = await invoke<any>("get_schema", {
        id: "translator_schema",
      });
      console.log("[loadSchema] Raw result:", JSON.stringify(result, null, 2));
      // Handle both wrapped (Response<T>) and unwrapped schema
      const schema = (result?.data ?? result) as SchemaPayload | null;
      if (!schema) {
        console.error("[loadSchema] No data returned:", result);
        ToastHelper.show("Schema not found in database", ToastKind.Error);
        return;
      }
      if (!schema.pages?.length) {
        console.error("[loadSchema] Schema has no pages:", schema);
        ToastHelper.show("Schema has no pages", ToastKind.Error);
        return;
      }
      const page = schema.pages[0];

      // Register components with SchemaRenderer
      this.schemaRenderer.setComponentResolver((selector) => ({
        selector,
        id: selector,
        name: selector,
        packageType: "ui",
        category: "general",
        props: {},
        defaultClasses: "",
      } as any));

      // Load schema into renderer
      this.schemaRenderer.loadSchema({ pages: [page] } as any);

      // Render into container
      if (this.schemaContainer) {
        await this.schemaRenderer.render(
          this.schemaContainer.nativeElement,
          {
            id: page.id,
            name: page.meta?.title || page.id,
            elements: page.canvasElements || [],
          } as any,
        );
      }

      // Load shortcuts from schema dynamically
      if (schema?.shortcuts?.length) {
        this.shortcutService.loadFromSchema(schema.shortcuts);
      }

      // Store schema reference for theme init in ngOnInit
      (this as any)._schema = schema;
    } catch (err) {
      console.error("Failed to load schema:", err);
      ToastHelper.show("Failed to load UI schema", ToastKind.Error);
    }
  }

  private setupEventBridge() {
    // Shortcuts button → toggle shortcuts overlay
    this.eventBusUnsubs.push(
      this.eventBus.on("shortcuts-btn:onShortcutsOpen", () =>
        this.toggleShortcuts(),
      ),
    );

    // App language selector → change UI locale
    this.eventBusUnsubs.push(
      this.eventBus.on("lang-selector:onAppLangChange", () =>
        this.onAppLangChange(),
      ),
    );

    // Overlay close button clicked → close overlay
    this.eventBusUnsubs.push(
      this.eventBus.on("shortcuts-el:close", () => {
        this.showShortcuts = false;
        this._setOverlayOpen(false);
      }),
    );

    // Theme toggle
    this.eventBusUnsubs.push(
      this.eventBus.on("theme-btn:onThemeToggle", () => this.onThemeToggle()),
    );

    // Source text input → schedule translation
    this.eventBusUnsubs.push(
      this.eventBus.on(
        "source-input:onInputTextChange",
        (data: any) => {
          const value = data?.event?.detail?.value ?? "";
          this.inputText = value;
          this.scheduleTranslation();
        },
      ),
    );

    // Source input clear
    this.eventBusUnsubs.push(
      this.eventBus.on("source-input:onClearInput", () => this.onClearInput()),
    );

    // Source language change
    this.eventBusUnsubs.push(
      this.eventBus.on(
        "source-lang:onSourceLangChange",
        (data: any) => {
          const value = data?.event?.detail?.value ?? "";
          this.sourceLang = value;
          GlobalStateService.instance.setSourceLang(value);
          this.scheduleTranslation();
        },
      ),
    );

    // Target language change
    this.eventBusUnsubs.push(
      this.eventBus.on(
        "target-lang:onTargetLangChange",
        (data: any) => {
          const value = data?.event?.detail?.value ?? "";
          this.targetLang = value;
          GlobalStateService.instance.setTargetLang(value);
          this.scheduleTranslation();
        },
      ),
    );

    // Swap languages
    this.eventBusUnsubs.push(
      this.eventBus.on("swap-btn:onSwapLanguages", () =>
        this.onSwapLanguages(),
      ),
    );

    // Translate button
    this.eventBusUnsubs.push(
      this.eventBus.on("translate-btn:onTranslate", () => this.onTranslate()),
    );

    // Copy translation
    this.eventBusUnsubs.push(
      this.eventBus.on("target-output:onCopyTranslation", () =>
        this.onCopyTranslation(),
      ),
    );

    // ShortcutService fires "onShortcutsOpen" event when F1 is pressed
    this.eventBusUnsubs.push(
      this.eventBus.on("onShortcutsOpen", () => this.toggleShortcuts()),
    );
  }

  @HostListener("window:keydown", ["$event"])
  handleKeyboardShortcut(e: KeyboardEvent) {
    if (e.key === "F1" || ((e.ctrlKey || e.metaKey) && e.key === "/")) {
      e.preventDefault();
      this.showShortcuts = !this.showShortcuts;
      this._setOverlayOpen(this.showShortcuts);
    }
    if (e.key === "Escape" && this.showShortcuts) {
      this.showShortcuts = false;
      this._setOverlayOpen(false);
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      this.onTranslate();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "l") {
      e.preventDefault();
      this.onSwapLanguages();
    }
  }

  private setupKeyboardShortcuts() {
    // Keyboard shortcut handling is done via @HostListener above
  }

  private async loadLanguages() {
    try {
      this.languages = await this.translationService.getSupportedLanguages();
      if (this.languages.length > 0) {
        this.sourceLang = this.languages[0].code;
        GlobalStateService.instance.setSourceLang(this.languages[0].code);
        // Default target to second language or first non-source
        const target = this.languages.length > 1 ? this.languages[1].code : this.languages[0].code;
        this.targetLang = target;
        GlobalStateService.instance.setTargetLang(target);
      }
    } catch {
      ToastHelper.show("Failed to load languages", ToastKind.Error);
    }
  }

  private async setupTranslationListener() {
    await listen<any>("translation-result", (event) => {
      const { requestId, response } = event.payload;
      if (
        this.currentRequestId !== null &&
        requestId === this.currentRequestId
      ) {
        this.currentRequestId = null;
        this.emitToOutput("loading-complete");
        if (response.status === "error") {
          this.emitToOutput("show-error", response.message);
        } else if (response.data?.translatedText) {
          this.translatedText = response.data.translatedText;
          this.emitToOutput("update-output", this.translatedText);
        }
      }
    });
  }

  onSourceLangChange(event: Event) {
    const value = (event as any).detail?.value ?? (event as any).value ?? "";
    this.sourceLang = value;
    this.scheduleTranslation();
  }

  onTargetLangChange(event: Event) {
    const value = (event as any).detail?.value ?? (event as any).value ?? "";
    this.targetLang = value;
    this.scheduleTranslation();
  }

  onInputTextChange(event: Event) {
    const value = (event as any).detail?.value ?? (event as any).value ?? "";
    this.inputText = value;
    this.scheduleTranslation();
  }

  onClearInput() {
    this.inputText = "";
    this.translatedText = "";
    this.cancelPending();
    this.emitToOutput("update-output", "");
    // Close shortcuts overlay if open
    if (this.showShortcuts) {
      this.showShortcuts = false;
      this._setOverlayOpen(false);
    }
    ToastHelper.show("Text cleared", ToastKind.Info);
  }

  onSwapLanguages() {
    const tempLang = this.sourceLang;
    this.sourceLang = this.targetLang;
    this.targetLang = tempLang;
    GlobalStateService.instance.swapLanguages();

    const tempText = this.inputText;
    this.inputText = this.translatedText;
    this.translatedText = tempText;

    this.emitToOutput("update-input", this.inputText);
    this.emitToOutput("update-output", this.translatedText);
    this.scheduleTranslation();
  }

  onTranslate() {
    this.triggerTranslation();
  }

  onCopyTranslation() {
    if (!this.translatedText) {
      ToastHelper.show("Nothing to copy", ToastKind.Info);
      return;
    }
    navigator.clipboard.writeText(this.translatedText).then(() => {
      ToastHelper.show("Copied to clipboard!", ToastKind.Success);
    }).catch(() => {
      ToastHelper.show("Failed to copy", ToastKind.Error);
    });
  }

  onThemeToggle() {
    this.themeService.toggleDarkMode();
  }

  onAppLangChange() {
    const selector = document.querySelector('[data-element-id="lang-selector"]') as any;
    if (selector?.value) {
      const locale = selector.value as 'en' | 'ru';
      I18nService.instance.setLocale(locale);
      GlobalStateService.instance.setAppLocale(locale);
    }
  }

  toggleShortcuts() {
    this.showShortcuts = !this.showShortcuts;
    this._setOverlayOpen(this.showShortcuts);
  }

  private _setOverlayOpen(open: boolean): void {
    const overlay = document.querySelector('[data-element-id="shortcuts-el"]') as any;
    if (overlay && typeof overlay.open === "boolean") {
      overlay.open = open;
    }
  }

  private async triggerTranslation() {
    const text = this.inputText.trim();
    if (!text) {
      this.translatedText = "";
      this.emitToOutput("update-output", "");
      return;
    }
    try {
      const requestId = await invoke<number>("translate_text", {
        text,
        source_lang: this.sourceLang,
        target_lang: this.targetLang,
      });
      if (requestId !== null && requestId !== undefined) {
        this.currentRequestId = requestId;
        this.emitToOutput("loading-start");
      } else {
        ToastHelper.show("Translation failed", ToastKind.Error);
      }
    } catch (e: any) {
      ToastHelper.show(e?.message || "Translation failed", ToastKind.Error);
    }
  }

  private scheduleTranslation() {
    this.cancelPending();
    const text = this.inputText.trim();
    if (!text) return;
    this.debounceTimer = setTimeout(() => this.triggerTranslation(), 500);
  }

  private _applyThemeState() {
    // Persist dark mode preference
    const isDark = this.themeService.isDarkMode();
    localStorage.setItem("dark_mode", String(isDark));
  }

  private cancelPending() {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
  }

  private emitToOutput(event: string, data?: any) {
    const output = document.getElementById("outputText");
    if (output) {
      output.dispatchEvent(new CustomEvent(event, { detail: data, bubbles: true }));
    }
    const input = document.getElementById("inputText");
    if (input && event === "update-input") {
      (input as any).value = data;
      input.dispatchEvent(new CustomEvent("input", { detail: { value: data }, bubbles: true }));
    }
  }
}

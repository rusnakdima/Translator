/* sys lib */
import { Component, inject, OnInit, CUSTOM_ELEMENTS_SCHEMA } from "@angular/core";
import { CommonModule } from "@angular/common";
import {
  SchemaRouterService,
  SchemaRouteViewerComponent,
  ComponentRegistryService,
  InvokeWrapperService,
  ThemeService,
  EventBusService,
} from "@tauri-front/shared";

/* services */
import { TranslationService } from "@features/translation/services/translation.service";

/* helpers */
import { ToastHelper } from "@shared/utils/toast.helper";

/* constants */
import { ToastKind } from "@shared/utils/constants";
import { SHORTCUTS } from "@shared/utils/constants";

/* tauri */
import { listen } from "@tauri-apps/api/event";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [CommonModule, SchemaRouteViewerComponent],
  schemas: [CUSTOM_ELEMENTS_SCHEMA],
  templateUrl: "./app.html",
})
export class App implements OnInit {
  private schemaRouter = inject(SchemaRouterService);
  private componentRegistry = inject(ComponentRegistryService);
  private invokeWrapper = new InvokeWrapperService();
  private themeService = inject(ThemeService);
  private eventBus = inject(EventBusService);
  private translationService = inject(TranslationService);

  private languages: Array<{ code: string; name: string }> = [];
  private sourceLang = "";
  private targetLang = "es";
  private inputText = "";
  private translatedText = "";
  private debounceTimer: ReturnType<typeof setTimeout> | null = null;
  private currentRequestId: number | null = null;

  private showShortcuts = false;

  ngOnInit() {
    this.registerComponents();
    this.loadSchema();
    this.setupEventHandlers();
    this.setupToastBridge();
    this.loadLanguages();
    this.setupTranslationListener();
    this.setupKeyboardShortcuts();
  }

  private registerComponents() {
    this.componentRegistry.register("language-selector", {
      selector: "app-language-selector",
    });
    this.componentRegistry.register("text-input", {
      selector: "app-text-input",
    });
    this.componentRegistry.register("swap-button", {
      selector: "app-swap-button",
    });
    this.componentRegistry.register("translation-output", {
      selector: "app-translation-output",
    });
    this.componentRegistry.register("shortcuts-overlay", {
      selector: "app-shortcuts-overlay",
    });
    this.componentRegistry.register("theme-toggle", {
      selector: "app-theme-toggle",
    });
    this.componentRegistry.register("button", { selector: "app-button" });
    this.componentRegistry.register("loading", { selector: "app-loading" });
    this.componentRegistry.register("text", { selector: "app-text" });
    // Layout elements — plain HTML tags
    this.componentRegistry.register("div", { selector: "div" });
    this.componentRegistry.register("span", { selector: "span" });
    this.componentRegistry.register("p", { selector: "p" });
    this.componentRegistry.register("h1", { selector: "h1" });
    this.componentRegistry.register("h2", { selector: "h2" });
    this.componentRegistry.register("footer", { selector: "footer" });
    this.componentRegistry.register("h1", { selector: "h1" });
  }

  private async loadSchema() {
    try {
      const response = await this.invokeWrapper.invoke<any>(
        "get_translator_schema",
      );
      const schema = response?.data ?? response;
      if (schema) {
        this.schemaRouter.setSchema(schema);
        this.schemaRouter.navigate("");
      }
    } catch (e) {
      console.error("Failed to load schema:", e);
    }
  }

  private setupToastBridge() {
    window.showToast = (
      msg: string,
      type = ToastKind.Info,
      duration = 3000,
    ) => {
      this.eventBus.showToast(msg, type, duration);
    };
    ToastHelper.setEventBus(this.eventBus);
  }

  private async loadLanguages() {
    try {
      this.languages = await this.translationService.getSupportedLanguages();
      if (this.languages.length > 0) {
        this.sourceLang = this.languages[0].code;
      }
      this.eventBus.emit("languages-loaded", this.languages);
    } catch {
      ToastHelper.show("Failed to load languages", ToastKind.Error);
    }
  }

  private setupEventHandlers() {
    // Theme toggle from app-theme-toggle component
    this.eventBus.on("onThemeToggle", () => this.themeService.toggle());

    // Shortcuts overlay
    this.eventBus.on("onShortcutsOpen", () => {
      this.showShortcuts = !this.showShortcuts;
      this.eventBus.emit("shortcuts-state", { open: this.showShortcuts, shortcuts: SHORTCUTS });
    });
    this.eventBus.on("onShortcutsClose", () => {
      this.showShortcuts = false;
      this.eventBus.emit("shortcuts-state", { open: false, shortcuts: SHORTCUTS });
    });

    // Language change events — detail contains {value: string}
    this.eventBus.on("onSourceLangChange", (data: any) => {
      const value = data?.detail?.value ?? data?.value ?? "";
      this.sourceLang = value;
      this.scheduleTranslation();
    });

    this.eventBus.on("onTargetLangChange", (data: any) => {
      const value = data?.detail?.value ?? data?.value ?? "";
      this.targetLang = value;
      this.scheduleTranslation();
    });

    // Text input — detail contains {value: string}
    this.eventBus.on("onInputTextChange", (data: any) => {
      const value = data?.detail?.value ?? data?.value ?? "";
      this.inputText = value;
      this.scheduleTranslation();
    });

    this.eventBus.on("onClearInput", () => {
      this.inputText = "";
      this.translatedText = "";
      this.cancelPending();
      this.eventBus.emit("update-output", "");
      ToastHelper.show("Text cleared", ToastKind.Info);
    });

    this.eventBus.on("onSwapLanguages", () => {
      const tempLang = this.sourceLang;
      this.sourceLang = this.targetLang;
      this.targetLang = tempLang;

      const tempText = this.inputText;
      this.inputText = this.translatedText;
      this.translatedText = tempText;

      this.eventBus.emit("update-input", this.inputText);
      this.eventBus.emit("update-output", this.translatedText);
      this.scheduleTranslation();
    });

    this.eventBus.on("onTranslate", () => this.triggerTranslation());

    this.eventBus.on("onCopyTranslation", async () => {
      if (!this.translatedText) {
        ToastHelper.show("Nothing to copy", ToastKind.Info);
        return;
      }
      try {
        await navigator.clipboard.writeText(this.translatedText);
        ToastHelper.show("Copied to clipboard!", ToastKind.Success);
      } catch {
        ToastHelper.show("Failed to copy", ToastKind.Error);
      }
    });
  }

  private async setupTranslationListener() {
    await listen<any>("translation-result", (event) => {
      const { requestId, response } = event.payload;
      if (
        this.currentRequestId !== null &&
        requestId === this.currentRequestId
      ) {
        this.currentRequestId = null;
        this.eventBus.emit("loading-complete");
        if (response.status === "error") {
          this.eventBus.emit("show-error", response.message);
        } else if (response.data?.translatedText) {
          this.translatedText = response.data.translatedText;
          this.eventBus.emit("update-output", this.translatedText);
        }
      }
    });
  }

  private async triggerTranslation() {
    const text = this.inputText.trim();
    if (!text) {
      this.translatedText = "";
      this.eventBus.emit("update-output", "");
      return;
    }
    try {
      const requestId = await this.invokeWrapper.invoke<number>(
        "translate_text",
        {
          text,
          source_lang: this.sourceLang,
          target_lang: this.targetLang,
        },
      );
      this.currentRequestId = requestId;
      this.eventBus.emit("loading-start");
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

  private setupKeyboardShortcuts() {
    window.addEventListener("keydown", (e: KeyboardEvent) => {
      if (e.key === "F1" || ((e.ctrlKey || e.metaKey) && e.key === "/")) {
        e.preventDefault();
        this.showShortcuts = !this.showShortcuts;
        this.eventBus.emit("shortcuts-state", { open: this.showShortcuts, shortcuts: SHORTCUTS });
      }
      if (e.key === "Escape" && this.showShortcuts) {
        this.showShortcuts = false;
        this.eventBus.emit("shortcuts-state", { open: false, shortcuts: SHORTCUTS });
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        this.triggerTranslation();
      }
      if ((e.ctrlKey || e.metaKey) && e.key === "l") {
        e.preventDefault();
        this.eventBus.emit("onSwapLanguages");
      }
    });
  }

  private cancelPending() {
    if (this.debounceTimer) {
      clearTimeout(this.debounceTimer);
      this.debounceTimer = null;
    }
  }
}

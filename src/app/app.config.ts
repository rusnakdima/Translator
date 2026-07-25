/* sys lib */
import {
  ApplicationConfig,
  APP_INITIALIZER,
  provideBrowserGlobalErrorListeners,
  provideZoneChangeDetection,
  inject,
  Injector,
} from "@angular/core";
import { registerAllAsCustomElements } from "@tauri-front/shared";

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideZoneChangeDetection({ eventCoalescing: true }),
    {
      provide: APP_INITIALIZER,
      useFactory: () => {
        const injector = inject(Injector);
        return () => registerAllAsCustomElements(injector);
      },
      multi: true,
    },
  ],
};

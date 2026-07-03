// Static import BEFORE bootstrap - ensures services are registered before any inject() calls
import "@tauri-front/shared";

import { bootstrapApplication } from "@angular/platform-browser";
import { App } from "./app/app";
import { appConfig } from "./app/app.config";

bootstrapApplication(App, appConfig)
  .catch((err) => console.error(err));

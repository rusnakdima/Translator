import { bootstrapApplication } from "@angular/platform-browser";
import { App } from "./app/app";
import { appConfig } from "./app/app.config";

bootstrapApplication(App, appConfig)
  .then(() => {
    import("@tauri-front/shared");
  })
  .catch((err) => console.error(err));

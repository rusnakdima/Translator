/* sys lib */
import { Component } from "@angular/core";

/* library */
import { SchemaShellComponent } from "@tauri-front/shared";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [SchemaShellComponent],
  template: `<lib-schema-shell appId="translator" />`,
})
export class App {}

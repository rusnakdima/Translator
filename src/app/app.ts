/* sys lib */
import { Component } from "@angular/core";

/* components */
import { TranslationComponent } from "@views/translation/translation.component";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [TranslationComponent],
  templateUrl: "./app.html",
})
export class App {}

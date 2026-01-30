/* sys lib */
import { Component, input, output, model } from "@angular/core";
import { FormsModule } from "@angular/forms";

@Component({
  selector: "app-text-input",
  standalone: true,
  imports: [FormsModule],
  templateUrl: "./text-input.component.html",
})
export class TextInputComponent {
  inputId = input.required<string>();
  placeholder = input.required<string>();
  text = model.required<string>();
  charCount = input.required<string>();
  keyDown = output<KeyboardEvent>();
  textChange = output<string>();
  clear = output<void>();

  onTextChange(): void {
    this.textChange.emit(this.text());
  }

  onClear(): void {
    this.text.set("");
    this.textChange.emit("");
    this.clear.emit();
  }
}

/* sys lib */
import {
  Component,
  input,
  output,
  model,
  ViewChild,
  ElementRef,
} from "@angular/core";
import { FormsModule } from "@angular/forms";
import { AutoResizeDirective } from "@directives/auto-resize.directive";
import { AppIconComponent } from "@components/icons/app-icon.component";

@Component({
  selector: "app-text-input",
  standalone: true,
  imports: [FormsModule, AutoResizeDirective, AppIconComponent],
  templateUrl: "./text-input.component.html",
})
export class TextInputComponent {
  @ViewChild("textArea") textAreaRef!: ElementRef<HTMLTextAreaElement>;

  inputId = input.required<string>();
  placeholder = input.required<string>();
  text = model.required<string>();
  charCount = input.required<string>();
  keyDown = output<KeyboardEvent>();
  textChange = output<string>();
  clear = output<void>();

  focus(): void {
    this.textAreaRef?.nativeElement.focus();
  }

  onTextChange(): void {
    this.textChange.emit(this.text());
  }

  onClear(): void {
    this.text.set("");
    this.textChange.emit("");
    this.clear.emit();
  }
}

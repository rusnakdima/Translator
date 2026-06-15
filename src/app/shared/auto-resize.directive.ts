import {
  Directive,
  ElementRef,
  HostListener,
  input,
  inject,
} from "@angular/core";

@Directive({
  selector: "[appAutoResize]",
  standalone: true,
})
export class AutoResizeDirective {
  maxHeight = input<number>(Infinity);
  minHeight = input<number>(0);

  private elementRef = inject(ElementRef<HTMLTextAreaElement>);

  @HostListener("input")
  onInput(): void {
    this.resize();
  }

  private resize(): void {
    const el = this.elementRef.nativeElement;
    el.style.height = "auto";
    const newHeight = Math.max(
      this.minHeight(),
      Math.min(el.scrollHeight, this.maxHeight()),
    );
    el.style.height = `${newHeight}px`;
  }
}

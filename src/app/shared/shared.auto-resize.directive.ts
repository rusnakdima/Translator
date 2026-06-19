import {
  Directive,
  ElementRef,
  HostListener,
  OnInit,
  OnDestroy,
} from "@angular/core";

@Directive({
  selector: "[appAutoResize]",
  standalone: true,
})
export class AutoResizeDirective implements OnInit, OnDestroy {
  private textarea: HTMLTextAreaElement;
  private lastValue: string = "";
  private checkInterval?: ReturnType<typeof setInterval>;

  constructor(private el: ElementRef<HTMLTextAreaElement>) {
    this.textarea = this.el.nativeElement;
  }

  ngOnInit(): void {
    this.resize();

    // Poll for value changes (works with ngModel on readonly elements)
    this.checkInterval = setInterval(() => {
      const currentValue = this.textarea.value;
      if (currentValue !== this.lastValue) {
        this.lastValue = currentValue;
        this.resize();
      }
    }, 100);
  }

  ngOnDestroy(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
    }
  }

  @HostListener("input")
  onInput(): void {
    this.lastValue = this.textarea.value;
    this.resize();
  }

  @HostListener("window:resize")
  onWindowResize(): void {
    setTimeout(() => this.resize(), 0);
  }

  private resize(): void {
    this.textarea.style.height = "auto";
    this.textarea.style.height = `${this.textarea.scrollHeight}px`;
  }
}

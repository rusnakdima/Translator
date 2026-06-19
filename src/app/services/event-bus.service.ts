import { Injectable, signal, computed } from "@angular/core";

export interface EventPayload {
  [key: string]: unknown;
}

type EventHandler = (payload: EventPayload) => void;

@Injectable({
  providedIn: "root",
})
export class EventBusService {
  private handlers = new Map<string, Set<EventHandler>>();
  private eventHistory = signal<EventPayload[]>([]);
  private readonly maxHistory = 100;

  emit(eventName: string, payload: EventPayload = {}): void {
    const handlers = this.handlers.get(eventName);
    if (handlers) {
      handlers.forEach((handler) => handler(payload));
    }

    this.eventHistory.update((history) => {
      const newHistory = [...history, { eventName, ...payload }];
      if (newHistory.length > this.maxHistory) {
        return newHistory.slice(-this.maxHistory);
      }
      return newHistory;
    });
  }

  on(eventName: string, handler: EventHandler): () => void {
    if (!this.handlers.has(eventName)) {
      this.handlers.set(eventName, new Set());
    }
    this.handlers.get(eventName)!.add(handler);

    return () => {
      this.handlers.get(eventName)?.delete(handler);
    };
  }

  once(eventName: string, handler: EventHandler): () => void {
    const wrapper: EventHandler = (payload) => {
      handler(payload);
      this.off(eventName, wrapper);
    };
    return this.on(eventName, wrapper);
  }

  off(eventName: string, handler: EventHandler): void {
    this.handlers.get(eventName)?.delete(handler);
  }

  clear(eventName?: string): void {
    if (eventName) {
      this.handlers.delete(eventName);
    } else {
      this.handlers.clear();
    }
  }

  history = computed(() => this.eventHistory());
}

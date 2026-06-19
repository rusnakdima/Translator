import { Injectable, inject } from "@angular/core";
import { StorageCacheService } from "./storage-cache.service";

export interface StorageQueryOptions {
  cache?: boolean;
  cacheTTL?: number;
}

@Injectable({
  providedIn: "root",
})
export class StorageQueryService {
  private readonly cache = inject(StorageCacheService);

  async query<T>(
    storageKey: string,
    fetcher: () => Promise<T>,
    options: StorageQueryOptions = {},
  ): Promise<T> {
    const { cache: useCache = true, cacheTTL } = options;

    if (useCache && this.cache.has(storageKey)) {
      const cached = this.cache.get<T>(storageKey);
      if (cached !== null) return cached;
    }

    const value = await fetcher();

    if (useCache) {
      this.cache.set(storageKey, value, cacheTTL);
    }

    return value;
  }

  invalidate(storageKey: string): void {
    this.cache.delete(storageKey);
  }

  invalidatePattern(pattern: string): void {
    const prefix = pattern.replace(/\*/g, "");
    for (const key of this.cache["cache"].keys()) {
      if (key.startsWith(prefix)) {
        this.cache.delete(key);
      }
    }
  }
}

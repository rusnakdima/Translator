import { Injectable, inject } from "@angular/core";
import { StorageService } from "./storage.service";
import { StorageCacheService } from "./storage-cache.service";
import { StorageQueryService } from "./storage-query.service";

@Injectable({
  providedIn: "root",
})
export class UnifiedStorageService {
  private readonly storage = inject(StorageService);
  private readonly cache = inject(StorageCacheService);
  private readonly query = inject(StorageQueryService);

  get storageService(): StorageService {
    return this.storage;
  }

  get cacheService(): StorageCacheService {
    return this.cache;
  }

  get queryService(): StorageQueryService {
    return this.query;
  }
}

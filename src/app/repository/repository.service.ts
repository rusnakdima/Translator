import { Injectable } from "@angular/core";
import { Observable } from "rxjs";

export interface RepositoryService<T> {
  getAll(): Observable<T[]>;
  getById(id: string): Observable<T | null>;
  create(data: Partial<T>): Observable<T>;
  update(id: string, data: Partial<T>): Observable<T>;
  delete(id: string): Observable<void>;
}

@Injectable({ providedIn: "root" })
export abstract class BaseRepository<T> implements RepositoryService<T> {
  abstract getAll(): Observable<T[]>;
  abstract getById(id: string): Observable<T | null>;
  abstract create(data: Partial<T>): Observable<T>;
  abstract update(id: string, data: Partial<T>): Observable<T>;
  abstract delete(id: string): Observable<void>;

  protected cacheKey(id?: string): string {
    return id
      ? `${this.constructor.name}:${id}`
      : `${this.constructor.name}:all`;
  }
}

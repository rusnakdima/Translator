import { Injectable } from "@angular/core";

const AUTH_TOKEN_KEY = "auth_token";

@Injectable({ providedIn: "root" })
export class StorageService {
  setAuthToken(token: string): void {
    localStorage.setItem(AUTH_TOKEN_KEY, token);
  }

  getAuthToken(): string | null {
    return localStorage.getItem(AUTH_TOKEN_KEY);
  }

  removeAuthToken(): void {
    localStorage.removeItem(AUTH_TOKEN_KEY);
  }

  clear(): void {
    localStorage.clear();
  }
}

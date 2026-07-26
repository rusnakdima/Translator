import { describe, it, expect, vi, beforeEach } from "vitest";
import { ToastHelper } from "./toast";

vi.mock("./constants", async () => {
  const actual = await vi.importActual("./constants");
  return {
    ...actual,
    ToastKind: {
      Info: "info",
      Success: "success",
      Error: "error",
    },
  };
});

describe("ToastHelper", () => {
  beforeEach(() => {
    ToastHelper.setToastService(null);
  });

  describe("setToastService", () => {
    it("should accept a toast service object", () => {
      const mockService = {
        info: vi.fn(),
        success: vi.fn(),
        error: vi.fn(),
      };
      ToastHelper.setToastService(mockService);
      expect(ToastHelper).toBeDefined();
    });
  });

  describe("show", () => {
    it("should call info on service when type is info", () => {
      const mockService = { info: vi.fn(), success: vi.fn(), error: vi.fn() };
      ToastHelper.setToastService(mockService);
      ToastHelper.show("hello", "info", 3000);
      expect(mockService.info).toHaveBeenCalledWith("hello", {
        duration: 3000,
      });
    });

    it("should call success on service when type is success", () => {
      const mockService = { info: vi.fn(), success: vi.fn(), error: vi.fn() };
      ToastHelper.setToastService(mockService);
      ToastHelper.show("done", "success", 2000);
      expect(mockService.success).toHaveBeenCalledWith("done", {
        duration: 2000,
      });
    });

    it("should call error on service when type is error", () => {
      const mockService = { info: vi.fn(), success: vi.fn(), error: vi.fn() };
      ToastHelper.setToastService(mockService);
      ToastHelper.show("failed", "error", 5000);
      expect(mockService.error).toHaveBeenCalledWith("failed", {
        duration: 5000,
      });
    });

    it("should default to info when no type given", () => {
      const mockService = { info: vi.fn(), success: vi.fn(), error: vi.fn() };
      ToastHelper.setToastService(mockService);
      ToastHelper.show("hello");
      expect(mockService.info).toHaveBeenCalledWith("hello", {
        duration: 3000,
      });
    });

    it("should default duration to 3000ms", () => {
      const mockService = { info: vi.fn(), success: vi.fn(), error: vi.fn() };
      ToastHelper.setToastService(mockService);
      ToastHelper.show("hello", "info");
      expect(mockService.info).toHaveBeenCalledWith("hello", {
        duration: 3000,
      });
    });
  });
});

export interface Response<T> {
  status: "success" | "info" | "warning" | "error";
  message: string;
  data: T;
}

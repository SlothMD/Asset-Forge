export interface WorkerJob {
  jobId: string;
  kind: "render-build" | "export-build";
  payload: unknown;
}

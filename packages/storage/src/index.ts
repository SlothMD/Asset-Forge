export interface LocalProjectStorageAdapter {
  openProject(path: string): Promise<unknown>;
  saveProject(projectId: string): Promise<void>;
}

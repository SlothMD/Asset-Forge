export type ProjectId = string;

export interface AssetForgeProject {
  schemaVersion: "0.1.0";
  projectId: ProjectId;
  name: string;
  createdAt: string;
  updatedAt: string;
}

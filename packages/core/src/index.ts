export type ProjectId = string;

export interface AssetForgeProject {
  schemaVersion: "0.1.0";
  projectId: ProjectId;
  name: string;
  githubRepositoryUrl: string;
  createdAt: string;
  updatedAt: string;
}

export interface MachineProjectBinding {
  projectId: ProjectId;
  localFolder: string;
  updatedAt: string;
}

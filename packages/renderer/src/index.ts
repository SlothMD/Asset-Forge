export interface RenderInput {
  templateId: string;
  dataRowId: string;
  exportPresetId: string;
}

export interface RenderArtifact {
  artifactId: string;
  path: string;
  format: "svg" | "png" | "pdf" | "json";
}

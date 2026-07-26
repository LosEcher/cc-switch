import type { FetchedModel } from "@/lib/api/model-fetch";
import type { CodexCatalogModel } from "@/types";

export function getImportableFetchedModels(
  fetchedModels: readonly FetchedModel[],
  catalogModels: readonly CodexCatalogModel[],
): FetchedModel[] {
  const seen = new Set(
    catalogModels.map((item) => item.model.trim()).filter(Boolean),
  );
  const importable: FetchedModel[] = [];

  for (const item of fetchedModels) {
    const id = item.id.trim();
    if (!id || seen.has(id)) continue;
    seen.add(id);
    importable.push({ ...item, id });
  }

  return importable;
}

export function mergeModelsIntoCodexCatalog(
  catalogModels: readonly CodexCatalogModel[],
  modelIds: readonly string[],
): CodexCatalogModel[] {
  const merged = [...catalogModels];
  const seen = new Set(
    catalogModels.map((item) => item.model.trim()).filter(Boolean),
  );

  for (const rawId of modelIds) {
    const id = rawId.trim();
    if (!id || seen.has(id)) continue;
    seen.add(id);
    merged.push({ model: id, displayName: id });
  }

  return merged;
}

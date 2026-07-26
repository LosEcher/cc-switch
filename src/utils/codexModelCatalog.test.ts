import { describe, expect, it } from "vitest";
import {
  getImportableFetchedModels,
  mergeModelsIntoCodexCatalog,
} from "@/utils/codexModelCatalog";

describe("getImportableFetchedModels", () => {
  it("returns unique fetched models that are not already mapped", () => {
    expect(
      getImportableFetchedModels(
        [
          { id: " gpt-5.6-sol ", ownedBy: "packy" },
          { id: "gpt-5.6-sol", ownedBy: "duplicate" },
          { id: "gpt-5.5", ownedBy: null },
          { id: "", ownedBy: null },
        ],
        [{ model: "gpt-5.5", displayName: "Existing" }],
      ),
    ).toEqual([{ id: "gpt-5.6-sol", ownedBy: "packy" }]);
  });
});

describe("mergeModelsIntoCodexCatalog", () => {
  it("imports models into an empty catalog", () => {
    expect(mergeModelsIntoCodexCatalog([], ["gpt-5.6-sol", "gpt-5.5"])).toEqual(
      [
        { model: "gpt-5.6-sol", displayName: "gpt-5.6-sol" },
        { model: "gpt-5.5", displayName: "gpt-5.5" },
      ],
    );
  });

  it("preserves existing order and hidden metadata while appending new models", () => {
    const existing = [
      {
        model: "gpt-5.5",
        displayName: "GPT 5.5",
        contextWindow: 200000,
        supportsParallelToolCalls: true,
        inputModalities: ["text", "image"],
        baseInstructions: "Keep this metadata",
      },
    ];

    const result = mergeModelsIntoCodexCatalog(existing, [
      "gpt-5.5",
      " gpt-5.6-sol ",
    ]);

    expect(result[0]).toBe(existing[0]);
    expect(result).toEqual([
      existing[0],
      { model: "gpt-5.6-sol", displayName: "gpt-5.6-sol" },
    ]);
  });

  it("ignores duplicate, empty, and whitespace-only IDs", () => {
    expect(
      mergeModelsIntoCodexCatalog([], ["gpt-5.6-sol", "", "  ", "gpt-5.6-sol"]),
    ).toEqual([{ model: "gpt-5.6-sol", displayName: "gpt-5.6-sol" }]);
  });
});

import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CodexModelCatalogImport } from "@/components/providers/forms/CodexModelCatalogImport";

Object.defineProperty(Element.prototype, "scrollIntoView", {
  configurable: true,
  value: vi.fn(),
});

describe("CodexModelCatalogImport", () => {
  it("imports the selected fetched models without offering existing mappings", () => {
    const onImport = vi.fn();
    render(
      <CodexModelCatalogImport
        fetchedModels={[
          { id: "gpt-5.5", ownedBy: "packy" },
          { id: "gpt-5.6-sol", ownedBy: "packy" },
          { id: "gpt-5.6-sol-mini", ownedBy: "packy" },
        ]}
        catalogModels={[{ model: "gpt-5.5", displayName: "GPT 5.5" }]}
        onImport={onImport}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "导入模型 (2)" }));

    expect(screen.queryByText("gpt-5.5")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("gpt-5.6-sol-mini"));
    fireEvent.click(screen.getByRole("button", { name: "导入所选 (1)" }));

    expect(onImport).toHaveBeenCalledWith(["gpt-5.6-sol"]);
  });

  it("shows a disabled completion state when every fetched model is mapped", () => {
    render(
      <CodexModelCatalogImport
        fetchedModels={[{ id: "gpt-5.5", ownedBy: "packy" }]}
        catalogModels={[{ model: "gpt-5.5" }]}
        onImport={vi.fn()}
      />,
    );

    expect(
      screen.getByRole("button", { name: "已全部加入映射" }),
    ).toBeDisabled();
  });
});

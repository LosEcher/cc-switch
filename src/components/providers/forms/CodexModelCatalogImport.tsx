import { useCallback, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { ListPlus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { FetchedModel } from "@/lib/api/model-fetch";
import type { CodexCatalogModel } from "@/types";
import { getImportableFetchedModels } from "@/utils/codexModelCatalog";

interface CodexModelCatalogImportProps {
  fetchedModels: readonly FetchedModel[];
  catalogModels: readonly CodexCatalogModel[];
  onImport: (modelIds: string[]) => void;
}

export function CodexModelCatalogImport({
  fetchedModels,
  catalogModels,
  onImport,
}: CodexModelCatalogImportProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [selectedModelIds, setSelectedModelIds] = useState<Set<string>>(
    new Set(),
  );
  const importableModels = useMemo(
    () => getImportableFetchedModels(fetchedModels, catalogModels),
    [catalogModels, fetchedModels],
  );

  const handleOpenChange = useCallback(
    (nextOpen: boolean) => {
      setOpen(nextOpen);
      if (nextOpen) {
        setSelectedModelIds(new Set(importableModels.map((model) => model.id)));
      }
    },
    [importableModels],
  );

  const handleToggleModel = useCallback((modelId: string) => {
    setSelectedModelIds((current) => {
      const next = new Set(current);
      if (next.has(modelId)) {
        next.delete(modelId);
      } else {
        next.add(modelId);
      }
      return next;
    });
  }, []);

  const selectedIds = importableModels
    .map((model) => model.id)
    .filter((id) => selectedModelIds.has(id));

  if (fetchedModels.length === 0) return null;

  const hasImportableModels = importableModels.length > 0;
  return (
    <Popover modal open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={!hasImportableModels}
          className="h-7 gap-1"
        >
          <ListPlus className="h-3.5 w-3.5" />
          {hasImportableModels
            ? t("codexConfig.importCatalogModels", {
                count: importableModels.length,
                defaultValue: "导入模型 ({{count}})",
              })
            : t("codexConfig.allFetchedModelsMapped", {
                defaultValue: "已全部加入映射",
              })}
        </Button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        sideOffset={6}
        collisionPadding={8}
        className="z-[1000] w-[min(360px,calc(100vw-2rem))] p-0"
      >
        <Command>
          <CommandInput
            placeholder={t("codexConfig.searchFetchedModels", {
              defaultValue: "搜索已获取的模型...",
            })}
          />
          <CommandList className="max-h-64">
            <CommandEmpty>
              {t("codexConfig.noMatchingFetchedModels", {
                defaultValue: "没有匹配的模型",
              })}
            </CommandEmpty>
            <CommandGroup>
              {importableModels.map((model) => (
                <CommandItem
                  key={model.id}
                  value={model.id}
                  keywords={model.ownedBy ? [model.ownedBy] : undefined}
                  onSelect={() => handleToggleModel(model.id)}
                >
                  <Checkbox
                    checked={selectedModelIds.has(model.id)}
                    tabIndex={-1}
                    aria-hidden="true"
                    className="pointer-events-none"
                  />
                  <span className="min-w-0 flex-1 truncate">{model.id}</span>
                  {model.ownedBy && (
                    <span className="max-w-24 truncate text-xs text-muted-foreground">
                      {model.ownedBy}
                    </span>
                  )}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
        <div className="flex flex-wrap items-center justify-between gap-2 border-t p-2">
          <div className="flex gap-1">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-7 px-2 text-xs"
              onClick={() =>
                setSelectedModelIds(
                  new Set(importableModels.map((model) => model.id)),
                )
              }
            >
              {t("codexConfig.selectAllFetchedModels", {
                defaultValue: "全选",
              })}
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="h-7 px-2 text-xs"
              onClick={() => setSelectedModelIds(new Set())}
            >
              {t("codexConfig.clearFetchedModelSelection", {
                defaultValue: "清空",
              })}
            </Button>
          </div>
          <Button
            type="button"
            size="sm"
            className="h-7"
            disabled={selectedIds.length === 0}
            onClick={() => {
              onImport(selectedIds);
              setOpen(false);
            }}
          >
            {t("codexConfig.importSelectedModels", {
              count: selectedIds.length,
              defaultValue: "导入所选 ({{count}})",
            })}
          </Button>
        </div>
      </PopoverContent>
    </Popover>
  );
}

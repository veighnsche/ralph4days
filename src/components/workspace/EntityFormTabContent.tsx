import type { LucideIcon } from "lucide-react";
import type { FieldValues, UseFormReturn } from "react-hook-form";
import { Button } from "@/components/ui/button";
import { Form } from "@/components/ui/form";
import { FormDescription, FormHeader, FormTitle } from "@/components/ui/form-header";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useTabMeta } from "@/hooks/useTabMeta";
import type { WorkspaceTab } from "@/stores/useWorkspaceStore";
import { useWorkspaceStore } from "@/stores/useWorkspaceStore";

export function EntityFormTabContent<T extends FieldValues>({
  tab,
  icon,
  entityName,
  form,
  onSubmit,
  isPending,
  children,
}: {
  tab: WorkspaceTab;
  icon: LucideIcon;
  entityName: string;
  form: UseFormReturn<T>;
  onSubmit: (data: T) => void;
  isPending: boolean;
  children: React.ReactNode;
}) {
  const mode = tab.data?.mode ?? "create";
  const label = mode === "create" ? `Create ${entityName}` : `Edit ${entityName}`;
  useTabMeta(tab.id, label, icon);
  const closeTab = useWorkspaceStore((s) => s.closeTab);

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="h-full flex flex-col">
        <div className="px-4 flex-shrink-0">
          <FormHeader>
            <FormTitle>{label}</FormTitle>
            <FormDescription>
              {mode === "create"
                ? `Add a new ${entityName.toLowerCase()} to your project`
                : `Update ${entityName.toLowerCase()} details`}
            </FormDescription>
          </FormHeader>
        </div>
        <Separator />
        <ScrollArea className="flex-1 min-h-0">
          <div className="px-4">{children}</div>
        </ScrollArea>
        <Separator />
        <div className="px-3 py-1.5 flex justify-end gap-2 flex-shrink-0">
          <Button type="button" variant="outline" size="default" onClick={() => closeTab(tab.id)} disabled={isPending}>
            Cancel
          </Button>
          <Button type="submit" size="default" disabled={isPending}>
            {isPending ? "Saving..." : mode === "create" ? "Create" : "Update"}
          </Button>
        </div>
      </form>
    </Form>
  );
}

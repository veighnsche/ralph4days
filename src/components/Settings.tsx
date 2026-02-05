import { Settings as SettingsIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Field, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { useTheme } from "@/lib/theme-provider";

export function Settings() {
  const { theme, setTheme } = useTheme();
  const isDark = theme === "dark";

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" size="icon">
          <SettingsIcon className="h-4 w-4" />
          <span className="sr-only">Settings</span>
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Customize your Ralph4days experience
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          <Field orientation="horizontal">
            <div className="flex-1">
              <FieldLabel>Dark Mode</FieldLabel>
              <FieldDescription>
                Use dark theme for the interface
              </FieldDescription>
            </div>
            <Switch
              checked={isDark}
              onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
            />
          </Field>
        </div>
      </DialogContent>
    </Dialog>
  );
}

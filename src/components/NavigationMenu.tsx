import { Layers, ListTodo, Menu, Target } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Page } from "@/hooks/useNavigation";

interface NavigationMenuProps {
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

export function NavigationMenu({ currentPage, onPageChange }: NavigationMenuProps) {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="h-10 w-10">
          <Menu className="h-5 w-5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <DropdownMenuLabel>Navigate to</DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => onPageChange("tasks")} disabled={currentPage === "tasks"}>
          <ListTodo className="mr-2 h-4 w-4" />
          Tasks
          {currentPage === "tasks" && <span className="ml-auto text-xs text-muted-foreground">•</span>}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => onPageChange("features")} disabled={currentPage === "features"}>
          <Target className="mr-2 h-4 w-4" />
          Features
          {currentPage === "features" && <span className="ml-auto text-xs text-muted-foreground">•</span>}
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => onPageChange("disciplines")} disabled={currentPage === "disciplines"}>
          <Layers className="mr-2 h-4 w-4" />
          Disciplines
          {currentPage === "disciplines" && <span className="ml-auto text-xs text-muted-foreground">•</span>}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

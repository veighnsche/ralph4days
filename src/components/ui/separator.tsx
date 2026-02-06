import { Separator as SeparatorPrimitive } from "radix-ui";
import type * as React from "react";

import { cn } from "@/lib/utils";

const BLEED_CONFIG = {
  sm: "-mx-2", // breaks out of px-2
  md: "-mx-4", // breaks out of px-4
  lg: "-mx-6", // breaks out of px-6
  xl: "-mx-8", // breaks out of px-8
};

function Separator({
  className,
  orientation = "horizontal",
  decorative = true,
  bleed,
  ...props
}: React.ComponentProps<typeof SeparatorPrimitive.Root> & {
  /** Negative margin to break out of parent padding. */
  bleed?: "sm" | "md" | "lg" | "xl";
}) {
  return (
    <SeparatorPrimitive.Root
      data-slot="separator"
      decorative={decorative}
      orientation={orientation}
      className={cn(
        "bg-border shrink-0 data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px",
        bleed && [BLEED_CONFIG[bleed], "data-[orientation=horizontal]:w-auto"],
        className
      )}
      {...props}
    />
  );
}

export { Separator };

import { cn } from "@/lib/utils";

interface FullBleedSeparatorProps {
  className?: string;
  /** Negative margin value to break out of parent padding. Default: -mx-4 (breaks out of px-4) */
  bleed?: "sm" | "md" | "lg" | "xl";
}

const BLEED_CONFIG = {
  sm: "-mx-2", // breaks out of px-2
  md: "-mx-4", // breaks out of px-4
  lg: "-mx-6", // breaks out of px-6
  xl: "-mx-8", // breaks out of px-8
};

export function FullBleedSeparator({ className, bleed = "md" }: FullBleedSeparatorProps) {
  return <hr className={cn("border-0 border-t border-border h-px shrink-0", BLEED_CONFIG[bleed], className)} />;
}

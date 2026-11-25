import type { ComponentProps } from "react";
import { Root } from "@radix-ui/react-label";

// shadcn/ui Label, adapted to the local Claude+ form styles.
export function Label(props: ComponentProps<typeof Root>) {
  return <Root data-slot="label" {...props} />;
}

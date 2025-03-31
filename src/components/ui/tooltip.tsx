import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import type { ReactNode } from "react";

export default function Tooltip({
	children,
	content,
}: { children: ReactNode; content: string }) {
	return (
		<TooltipPrimitive.Provider>
			<TooltipPrimitive.Root delayDuration={200}>
				<TooltipPrimitive.Trigger asChild>{children}</TooltipPrimitive.Trigger>
				<TooltipPrimitive.Portal>
					<TooltipPrimitive.Content
						className="bg-muted text-/80 text-sm px-3 py-1 rounded shadow-md"
						sideOffset={5}
					>
						{content}
					</TooltipPrimitive.Content>
				</TooltipPrimitive.Portal>
			</TooltipPrimitive.Root>
		</TooltipPrimitive.Provider>
	);
}

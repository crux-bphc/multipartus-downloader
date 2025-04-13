import { LogtoProvider } from "@/lib/logto";
import { Provider } from "jotai";
import { Suspense } from "react";
import { Outlet } from "react-router";
import { Toaster } from "@/components/ui/sonner";

export const AppLayout = () => {
	return (
		<Provider>
			<Suspense>
				<LogtoProvider>
					<Outlet />
					<Toaster />
				</LogtoProvider>
			</Suspense>
		</Provider>
	);
};

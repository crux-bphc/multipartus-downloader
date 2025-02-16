import { LogtoProvider } from "@/lib/logto";
import { Provider } from "jotai";
import { Suspense } from "react";
import { Outlet } from "react-router";

export const AppLayout = () => {
	return (
		<Provider>
			<Suspense>
				<LogtoProvider>
					<Outlet />
				</LogtoProvider>
			</Suspense>
		</Provider>
	);
};

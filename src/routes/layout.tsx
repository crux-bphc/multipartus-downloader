import { LogtoProvider } from "@/lib/logto";
import { Outlet } from "react-router";

export const AppLayout = () => {
	return (
		<LogtoProvider>
			<Outlet />
		</LogtoProvider>
	);
};

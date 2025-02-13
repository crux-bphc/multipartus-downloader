import { fetchLex } from "@/lib/lex";
import { LogtoProvider } from "@/lib/logto";
import { Outlet } from "react-router";
import { SWRConfig } from "swr";

export const AppLayout = () => {
	return (
		<LogtoProvider>
			<SWRConfig value={{ fetcher: fetchLex }}>
				<Outlet />
			</SWRConfig>
		</LogtoProvider>
	);
};

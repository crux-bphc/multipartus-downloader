import { fetchLex } from "@/lib/lex";
import { Outlet } from "react-router";
import { SWRConfig } from "swr";

export const AppLayout = () => {
	return (
		<SWRConfig value={{ fetcher: fetchLex }}>
			<Outlet />
		</SWRConfig>
	);
};

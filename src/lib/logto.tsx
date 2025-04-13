import LogtoClient, { createRequester, Prompt } from "@logto/browser";
import { fetch } from "@tauri-apps/plugin-http";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
	type ReactNode,
	createContext,
	useContext,
	useEffect,
	useState,
} from "react";
import { useNavigate } from "react-router";
import { toast } from "sonner";

export const logtoClient = new LogtoClient({
	endpoint: import.meta.env.VITE_LOGTO_ENDPOINT,
	appId: import.meta.env.VITE_LOGTO_APP_ID,
	scopes: ["openid", "email", "profile", "offline_access"],
	prompt: [Prompt.Consent],
});
logtoClient.adapter.navigate = (url) => openUrl(url);
logtoClient.adapter.requester = createRequester(fetch);

const LogtoContext = createContext<{
	isAuthenticated: boolean;
	updateAuthState: () => Promise<void>;
}>({
	isAuthenticated: false,
	updateAuthState: async () => {
		throw Error("Not implemented");
	},
});

export const LogtoProvider = ({ children }: { children?: ReactNode }) => {
	const [isAuthenticated, setIsAuthenticated] = useState<boolean | null>(null);
	const navigate = useNavigate();

	async function updateAuthState() {
		if (await logtoClient.isAuthenticated()) {
			await logtoClient.getAccessToken();
			setIsAuthenticated(true);
		} else {
			setIsAuthenticated(false);
		}
	}

	let loggedIn: string | number = "";
	let autoLogin = true;
	useEffect(() => {
		loggedIn = toast.info("Attempting to login automatically...");
		updateAuthState()
		.catch(() => {
			toast.error("Failed to login automatically!");
		});
	}, []);

	useEffect(() => {
		if (isAuthenticated) {
			navigate("/app");
			toast.dismiss(loggedIn);
		} else {
			if (autoLogin && isAuthenticated === false) {
				toast.error("You have not logged in yet. Please log in manually");
				autoLogin = false;
			}
			navigate("/");
		}
	}, [isAuthenticated, navigate]);

	return (
		<LogtoContext.Provider value={{ isAuthenticated: isAuthenticated ?? false, updateAuthState }}>
			{children}
		</LogtoContext.Provider>
	);
};

export const useLogto = () => useContext(LogtoContext);

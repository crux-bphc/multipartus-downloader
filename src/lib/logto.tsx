import LogtoClient, { Prompt } from "@logto/browser";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
	type ReactNode,
	createContext,
	useContext,
	useEffect,
	useState,
} from "react";
import { useNavigate } from "react-router";

const logtoClient = new LogtoClient({
	endpoint: import.meta.env.VITE_LOGTO_ENDPOINT,
	appId: import.meta.env.VITE_LOGTO_APP_ID,
	scopes: ["openid", "email", "profile", "offline_access"],
	prompt: [Prompt.Consent],
});
logtoClient.adapter.navigate = (url) => openUrl(url);

const LogtoContext = createContext<{
	logtoClient: LogtoClient;
	idToken?: string;
	isAuthenticated: boolean;
	updateAuthState: () => Promise<void>;
}>({
	logtoClient,
	idToken: undefined,
	isAuthenticated: false,
	updateAuthState: async () => {
		throw Error("Not implemented");
	},
});

export const LogtoProvider = ({ children }: { children?: ReactNode }) => {
	const [idToken, setIdToken] = useState<string | undefined>(undefined);
	const [isAuthenticated, setIsAuthenticated] = useState<boolean>(false);
	const navigate = useNavigate();

	async function updateAuthState() {
		setIsAuthenticated(await logtoClient.isAuthenticated());
	}

	async function updateIdToken() {
		setIdToken((await logtoClient.getIdToken()) ?? undefined);
	}

	useEffect(() => {
		updateAuthState();
	});

	useEffect(() => {
		updateIdToken();
		// Refresh id token every 30 minutes
		const id = setInterval(updateIdToken, 1000 * 60 * 30);
		return () => clearInterval(id);
	}, [isAuthenticated]);

	useEffect(() => {
		if (isAuthenticated) {
			navigate("/app");
		} else {
			navigate("/");
		}
	}, [isAuthenticated, navigate]);

	return (
		<LogtoContext.Provider
			value={{ logtoClient, idToken, isAuthenticated, updateAuthState }}
		>
			{children}
		</LogtoContext.Provider>
	);
};

export const useLogto = () => useContext(LogtoContext);

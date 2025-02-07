import LogtoClient from "@logto/browser";
import { openUrl } from "@tauri-apps/plugin-opener";

export const logtoClient = new LogtoClient({
	endpoint: import.meta.env.VITE_LOGTO_ENDPOINT,
	appId: import.meta.env.VITE_LOGTO_APP_ID,
});

logtoClient.adapter.navigate = (url) => openUrl(url);

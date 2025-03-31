import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import { DownloadPage } from "./routes/app";
import { LoginPage } from "./routes/index";
import "@fontsource-variable/lexend";
import "./app.css";
import { AppLayout } from "./routes/layout";

const node = document.getElementById("root");

if (!node) {
	throw new Error("No root element found");
}

// Should this be here? Does the app need maintain the facade of *not* being a react website?
node.addEventListener('contextmenu', (e: Event) => e.preventDefault())

createRoot(node).render(
	<BrowserRouter>
		<Routes>
			<Route element={<AppLayout />}>
				<Route index element={<LoginPage />} />
				<Route path="/app" element={<DownloadPage />} />
			</Route>
		</Routes>
	</BrowserRouter>,
);

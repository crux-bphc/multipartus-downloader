import { StrictMode } from "react";
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

createRoot(node).render(
	<StrictMode>
		<BrowserRouter>
			<Routes>
				<Route element={<AppLayout />}>
					<Route index element={<LoginPage />} />
					<Route path="/app" element={<DownloadPage />} />
				</Route>
			</Routes>
		</BrowserRouter>
	</StrictMode>,
);

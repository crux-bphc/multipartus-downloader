import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import { LoginPage } from "./routes";
import "@fontsource-variable/lexend";
import "./app.css";

const node = document.getElementById("root");

if (!node) {
	throw new Error("No root element found");
}

createRoot(node).render(
	<StrictMode>
		<BrowserRouter>
			<Routes>
				<Route path="/" element={<LoginPage />} />
			</Routes>
		</BrowserRouter>
	</StrictMode>,
);

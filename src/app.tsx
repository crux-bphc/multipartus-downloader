import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router";
import { HomePage } from "./routes";
import "./app.css";

const node = document.getElementById("root");

if (!node) {
	throw new Error("No root element found");
}

createRoot(node).render(
	<StrictMode>
		<BrowserRouter>
			<Routes>
				<Route path="/" element={<HomePage />} />
			</Routes>
		</BrowserRouter>
	</StrictMode>,
);

/* @refresh reload */
import { render } from "solid-js/web";
import "./app.css";

import { Router } from "@solidjs/router";
import { lazy } from "solid-js";

const routes = [
	{
		path: "/",
    component: lazy(() => import("./routes/index")),
	},
];

render(
	() => <Router>{routes}</Router>,
	document.getElementById("root") as HTMLElement,
);

import { Button } from "@/components/ui/button";
import { logtoClient, useLogto } from "@/lib/logto";
import { onUrl, start } from "@fabianlars/tauri-plugin-oauth";

function LoginButton() {
	const { updateAuthState } = useLogto();

	async function handleLogin() {
		const port = await start({ ports: [6942] });
		console.log(`oauth started at port ${port}`);
		await logtoClient.signIn(`http://localhost:${port}`);

		await onUrl(async (url) => {
			await logtoClient.handleSignInCallback(url);
			await updateAuthState();
		});
	}

	return (
		<Button onClick={handleLogin} size="lg" type="button">
			<svg
				xmlns="http://www.w3.org/2000/svg"
				width="1em"
				height="1em"
				viewBox="0 0 24 24"
			>
				<title>google</title>
				<path
					fill="currentColor"
					d="M12 2a9.96 9.96 0 0 1 6.29 2.226a1 1 0 0 1 .04 1.52l-1.51 1.362a1 1 0 0 1-1.265.06a6 6 0 1 0 2.103 6.836l.001-.004h-3.66a1 1 0 0 1-.992-.883L13 13v-2a1 1 0 0 1 1-1h6.945a1 1 0 0 1 .994.89q.06.55.061 1.11c0 5.523-4.477 10-10 10S2 17.523 2 12S6.477 2 12 2"
				/>
			</svg>
			Login with BITS Mail
		</Button>
	);
}

export function LoginPage() {
	return (
		<main className="mx-auto container px-4">
			<div className="flex flex-col items-center justify-center h-svh gap-12">
				<h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
					<span className="text-primary">MULTI</span>PARTUS Downloader
				</h1>
				<LoginButton />
				<p className="text-muted-foreground">
					Created by <span className="text-[#164a9e]">CRUx</span>, the coding
					and programming club of BITS Pilani, Hyderabad Campus
				</p>
			</div>
		</main>
	);
}

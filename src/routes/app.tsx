import { DownloadForm } from "@/components/download-form";
import { SettingsDialog } from "@/components/settings-dialog";
import { Button } from "@/components/ui/button";
import { fetchLex } from "@/lib/lex";
import { useAtom } from "jotai";
import { atomWithRefresh } from "jotai/utils";
import { RefreshCwIcon } from "lucide-react";

const validUserAtom = atomWithRefresh(async () => {
	const user = await fetchLex<{
		valid: boolean;
	}>("user");
	return user.valid;
});

export const DownloadPage = () => {
	const [isValid, refreshUser] = useAtom(validUserAtom);

	return (
		<main className="mx-auto container px-4">
			<SettingsDialog />
			{isValid ? (
				<DownloadForm />
			) : (
				<div className="flex flex-col gap-6 justify-center items-center h-screen text-center">
					<h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
						Invalid User
					</h1>
					<p className="text-xl text-muted-foreground">
						You are not registered to <b>Multipartus</b> on{" "}
						<span className="text-primary">Lex</span>.
					</p>
					<Button onClick={refreshUser}>
						Retry
						<RefreshCwIcon />
					</Button>
				</div>
			)}
		</main>
	);
};

import { DownloadForm } from "@/components/download-form";
import { SettingsDialog } from "@/components/settings-dialog";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogTitle } from "@/components/ui/dialog";
import { LoadingDots } from "@/components/ui/load-dots";
import { fetchLex } from "@/lib/lex";
import { useAtom } from "jotai";
import { atomWithRefresh } from "jotai/utils";
import { RefreshCwIcon } from "lucide-react";
import { Suspense } from "react";
import React, { ReactNode } from 'react';
import { Toaster } from "@/components/ui/sonner";

interface ErrorBoundaryProps {
	children: ReactNode;
	refresh: () => void;
}

interface ErrorBoundaryState {
	hasError: boolean;
}

class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
	constructor(props: ErrorBoundaryProps) {
		super(props);
		this.state = { hasError: false };
	}

	static getDerivedStateFromError(_: Error): ErrorBoundaryState {
		return { hasError: true };
	}

	render() {
		if (this.state.hasError) {
			return <NetworkError refresh={this.props.refresh}></NetworkError>;
		}
		return this.props.children;
	}
}

const validUserAtom = atomWithRefresh(async () => {
	const user = await fetchLex<{
		valid: boolean;
	}>("user");
	return user.valid
});	


const DownloadPageRoot = () => {
	const [isValid, refreshUser] = useAtom(validUserAtom);

	return (
		<main className="mx-auto container px-4">
			{isValid ? (
				<>
					<SettingsDialog />
					<DownloadForm />
					<Toaster />
				</>
			) : (
				<div className="flex flex-col gap-6 justify-center items-center h-screen text-center">
					<h1 className="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">
						Invalid User
					</h1>
					<p className="text-xl text-muted-foreground">
						You are not registered to <b>Multipartus</b> on{" "}
						<span className="text-primary">Lex</span>.
					</p>
					<Button onClick={() => refreshUser()}>
						Retry
						<RefreshCwIcon />
					</Button>
				</div>
				)
			}
		</main>
	);
}

export const DownloadPage = () => {
	return (
		// I didnt know how to handle this shit with states: everything I tried failed
		// so it's just gonna reload the window when it fails. I gave up on react
		<ErrorBoundary refresh={() => window.location.reload()}>
			<Suspense fallback={<Loading></Loading>}>
				<DownloadPageRoot></DownloadPageRoot>
			</Suspense>
		</ErrorBoundary>
	)
};


const NetworkError = ({ refresh }: { refresh: () => void }) => {
	return (
		<Dialog open={true}>
			<DialogContent>
				<DialogTitle>
					Something went wrong!
				</DialogTitle>
				It appears that your network has trouble connecting to our server!
				Try checking your connection and refresh
				<DialogFooter>
					<Button onClick={refresh}>
						Retry
						<RefreshCwIcon />
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}

const Loading = () => {
	return (
        <div className="flex justify-center items-center h-screen">
            <span className="-ml-4 text-3xl font-extrabold">LOADING<LoadingDots /></span>
        </div>
	);
}
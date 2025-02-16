import { DownloadForm } from "@/components/download-form";
import { Button } from "@/components/ui/button";
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from "@/components/ui/command";
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from "@/components/ui/popover";
import { subjectAtom } from "@/lib/atoms";
import { fetchLex } from "@/lib/lex";
import { useSetAtom } from "jotai";
import { useEffect, useState } from "react";
import { useDebounce } from "use-debounce";

function SearchSubject() {
	const setSelectedSubject = useSetAtom(subjectAtom);
	const [label, setLabel] = useState("Search subject");
	const [open, setOpen] = useState(false);
	const [search, setSearch] = useState("");
	const [debouncedSearch] = useDebounce(search, 500);
	const [subjects, setSubjects] = useState<Multipartus.Subject[]>([]);
	const [loading, setLoading] = useState(false);

	const formatSubject = (subject: Multipartus.Subject) =>
		`${subject.department} ${subject.code} - ${subject.name}`;

	useEffect(() => {
		setLoading(true);
		fetchLex<Multipartus.Subject[]>(
			`subject/search?q=${encodeURIComponent(debouncedSearch)}`,
		)
			.then(setSubjects)
			.finally(() => setLoading(false));
	}, [debouncedSearch]);

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button variant="outline" className="flex mx-auto w-full text-lg h-12">
					{label}
				</Button>
			</PopoverTrigger>
			<PopoverContent className="p-0 w-[var(--radix-popover-trigger-width)]">
				<Command shouldFilter={false}>
					<CommandInput value={search} onValueChange={setSearch} />
					<CommandList>
						<CommandEmpty>
							{loading ? "Fetching..." : "No subject found."}
						</CommandEmpty>
						<CommandGroup>
							{subjects?.map((subject) => (
								<CommandItem
									key={subject.id.ID.join()}
									value={subject.id.ID.join(";")}
									onSelect={() => {
										setSelectedSubject(subject.id.ID);
										setLabel(formatSubject(subject));
										setOpen(false);
									}}
								>
									{formatSubject(subject)}
								</CommandItem>
							))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	);
}

export const DownloadPage = () => {
	return (
		<main className="mx-auto container">
			<br />
			<SearchSubject />
			<DownloadForm />
			<br />
		</main>
	);
};

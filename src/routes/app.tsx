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
import { BirdIcon } from "lucide-react";
import { useState } from "react";
import useSWR from "swr";
import { useDebounce } from "use-debounce";

function SearchSubject(props: {
	selectSubject: (subject: [string, string]) => void;
}) {
	const [label, setLabel] = useState("Search subject");
	const [open, setOpen] = useState(false);
	const [search, setSearch] = useState("");
	const [debouncedSearch] = useDebounce(search, 500);
	const { data: subjects } = useSWR<Multipartus.Subject[]>(
		`subject/search?q=${encodeURIComponent(debouncedSearch)}`,
	);

	const formatSubject = (subject: Multipartus.Subject) =>
		`${subject.department} ${subject.code} - ${subject.name}`;

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button variant="outline" className="flex mx-auto w-lg text-lg">
					{label}
				</Button>
			</PopoverTrigger>
			<PopoverContent className="p-0 w-lg">
				<Command shouldFilter={false}>
					<CommandInput value={search} onValueChange={setSearch} />
					<CommandList>
						<CommandEmpty>No subject found.</CommandEmpty>
						<CommandGroup>
							{subjects?.map((subject) => (
								<CommandItem
									key={subject.id.ID.join()}
									value={subject.id.ID.join(";")}
									onSelect={() => {
										props.selectSubject(subject.id.ID);
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
	const [subject, setSubject] = useState<[string, string] | null>(null);
	return (
		<main className="mx-auto container">
			<br />
			<SearchSubject selectSubject={setSubject} />
			<br />
			{subject ? (
				<DownloadForm department={subject[0]} code={subject[1]} />
			) : (
				<div className="flex flex-col gap-6 justify-center items-center py-12">
					<BirdIcon className="w-64 h-64 text-muted-foreground" />
					<p className="leading-7">no subject selected</p>
				</div>
			)}
		</main>
	);
};

import { atom } from "jotai";

// selected subject
export const subjectAtom = atom<[string, string]>();

// selected lecture section for the selected subject
export const lectureAtom = atom<[number, number]>();

export interface Video extends Multipartus.Video {
	selected: boolean;
	index: number;
}

export const videosAtom = atom<Video[]>([]);

import { atom } from "jotai";

// selected subject
export const subjectAtom = atom<[string, string]>();

// selected lecture section for the selected subject
export const lectureAtom = atom<[number, number]>();

// list of videos for the selected lecture section
export const videosAtom = atom<Multipartus.Video[]>([]);

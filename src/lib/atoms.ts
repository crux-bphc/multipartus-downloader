import { atom } from "jotai";

// selected subject
export const subjectAtom = atom<[string, string]>();

// selected lecture section for the selected subject
export const lectureAtom = atom<[number, number]>();


export const videosAtom = atom<Multipartus.Video[]>([]);

import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { Book, BookWithChapters, LibraryRoot, ScanResult, Progress, AllProgress, AppSettings } from '$lib/types/book';

export async function getDataDir(): Promise<string> {
	return invoke<string>('get_data_dir');
}

export async function getAppMode(): Promise<string> {
	return invoke<string>('get_app_mode');
}

export async function scanLibrary(rootPath: string): Promise<ScanResult> {
	return invoke<ScanResult>('scan_library', { rootPath });
}

export async function getBooks(): Promise<Book[]> {
	return invoke<Book[]>('get_books');
}

export async function getBook(id: number): Promise<BookWithChapters> {
	return invoke<BookWithChapters>('get_book', { id });
}

export async function getLibraryRoots(): Promise<LibraryRoot[]> {
	return invoke<LibraryRoot[]>('get_library_roots');
}

export async function saveProgress(input: {
	book_id: number;
	position_ms: number;
	chapter_index: number;
	playback_speed: number;
	status: string;
}): Promise<void> {
	return invoke('save_progress', { input });
}

export async function getProgress(bookId: number): Promise<Progress | null> {
	return invoke<Progress | null>('get_progress', { bookId });
}

export async function getAllProgress(): Promise<AllProgress[]> {
	return invoke<AllProgress[]>('get_all_progress');
}

export async function getSettings(): Promise<AppSettings> {
	return invoke<AppSettings>('get_settings');
}

export async function saveSettings(settings: AppSettings): Promise<void> {
	return invoke('save_settings', { settings });
}

export interface SilenceRegion {
	start_ms: number;
	end_ms: number;
}

export async function analyzeSilence(
	filePath: string,
	thresholdDbfs: number,
	minGapMs: number,
): Promise<SilenceRegion[]> {
	return invoke<SilenceRegion[]>('analyze_silence', { filePath, thresholdDbfs, minGapMs });
}

/** Convert an absolute filesystem path to an asset protocol URL for the webview. */
export function assetUrl(path: string): string {
	return convertFileSrc(path);
}

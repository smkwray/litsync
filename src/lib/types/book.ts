export interface Book {
	id: number;
	title: string;
	author: string | null;
	narrator: string | null;
	description: string | null;
	total_duration_ms: number;
	cover_path: string | null;
	folder_path: string;
	file_count: number;
	added_at: string;
	updated_at: string;
}

export interface Chapter {
	id: number;
	book_id: number;
	title: string;
	start_ms: number;
	end_ms: number;
	file_path: string;
	file_index: number;
}

export interface BookWithChapters extends Book {
	chapters: Chapter[];
}

export interface LibraryRoot {
	id: number;
	path: string;
	last_scanned_at: string | null;
}

export interface ScanResult {
	books_found: number;
	books_added: number;
	errors: string[];
}

export interface Progress {
	book_id: number;
	position_ms: number;
	chapter_index: number;
	playback_speed: number;
	status: string;
	last_played_at: string | null;
	updated_at: string;
}

export type AppTheme = 'dark' | 'light';

export interface AppSettings {
	theme: AppTheme;
	default_speed: number;
	skip_short_sec: number;
	skip_long_sec: number;
	auto_rewind_enabled: boolean;
	auto_rewind_short_pause_sec: number;
	auto_rewind_short_rewind_sec: number;
	auto_rewind_long_pause_sec: number;
	auto_rewind_long_rewind_sec: number;
	sleep_timer_default_min: number;
	silence_skip_threshold: number;
	silence_skip_min_gap_ms: number;
}

export interface AllProgress {
	book_id: number;
	position_ms: number;
	total_duration_ms: number;
	status: string;
	playback_speed: number;
	last_played_at: string | null;
}

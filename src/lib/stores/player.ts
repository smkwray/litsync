import { writable, derived, get } from 'svelte/store';
import { audioEngine } from '$lib/utils/audio';
import { saveProgress as saveProgressApi, getProgress } from '$lib/api/tauri';
import { settingsStore } from '$lib/stores/settings';
import type { Chapter } from '$lib/types/book';

export interface PlayerState {
	bookId: number | null;
	bookTitle: string;
	author: string | null;
	coverPath: string | null;
	chapters: Chapter[];
	currentChapterIndex: number;
	isPlaying: boolean;
	currentTimeMs: number;
	totalDurationMs: number;
	playbackSpeed: number;
	volume: number;
	isLoaded: boolean;
	silenceSkip: boolean;
}

const initial: PlayerState = {
	bookId: null,
	bookTitle: '',
	author: null,
	coverPath: null,
	chapters: [],
	currentChapterIndex: 0,
	isPlaying: false,
	currentTimeMs: 0,
	totalDurationMs: 0,
	playbackSpeed: 1,
	volume: 1,
	isLoaded: false,
	silenceSkip: false,
};

export const playerState = writable<PlayerState>(initial);

export const currentChapter = derived(playerState, ($s) =>
	$s.chapters[$s.currentChapterIndex] ?? null
);

// Wire audio engine events to the store
audioEngine.on('timeupdate', ({ globalTimeMs }) => {
	playerState.update((s) => ({ ...s, currentTimeMs: globalTimeMs }));
});

let lastPausedAt = 0;

audioEngine.on('play', () => {
	playerState.update((s) => ({ ...s, isPlaying: true }));

	// Auto-rewind after pause
	if (lastPausedAt > 0) {
		const settings = get(settingsStore);
		if (settings.auto_rewind_enabled) {
			const pauseDuration = (Date.now() - lastPausedAt) / 1000;
			let rewindSec = 0;
			if (pauseDuration > settings.auto_rewind_long_pause_sec) {
				rewindSec = settings.auto_rewind_long_rewind_sec;
			} else if (pauseDuration > settings.auto_rewind_short_pause_sec) {
				rewindSec = settings.auto_rewind_short_rewind_sec;
			}
			if (rewindSec > 0) {
				audioEngine.seekRelative(-rewindSec * 1000);
			}
		}
		lastPausedAt = 0;
	}
});

audioEngine.on('pause', () => {
	playerState.update((s) => ({ ...s, isPlaying: false }));
	lastPausedAt = Date.now();
	persistProgress();
});

audioEngine.on('chapterchange', ({ index }) => {
	playerState.update((s) => ({ ...s, currentChapterIndex: index }));
	persistProgress();
});

audioEngine.on('ended', () => {
	playerState.update((s) => ({ ...s, isPlaying: false }));
	const s = get(playerState);
	if (s.bookId !== null) {
		saveProgressApi({
			book_id: s.bookId,
			position_ms: s.totalDurationMs,
			chapter_index: s.chapters.length - 1,
			playback_speed: s.playbackSpeed,
			status: 'finished',
		}).catch((e) => console.warn('Failed to save progress:', e));
	}
});

// -- Auto-save timer --

let saveInterval: ReturnType<typeof setInterval> | null = null;

function startAutoSave() {
	stopAutoSave();
	saveInterval = setInterval(() => {
		const s = get(playerState);
		if (s.isPlaying && s.bookId !== null) {
			persistProgress();
		}
	}, 5000);
}

function stopAutoSave() {
	if (saveInterval !== null) {
		clearInterval(saveInterval);
		saveInterval = null;
	}
}

function persistProgress() {
	const s = get(playerState);
	if (s.bookId === null) return;
	const ms = Math.round(s.currentTimeMs);
	console.log(`[progress] saving book=${s.bookId} pos=${ms}ms speed=${s.playbackSpeed}`);
	saveProgressApi({
		book_id: s.bookId,
		position_ms: ms,
		chapter_index: s.currentChapterIndex,
		playback_speed: s.playbackSpeed,
		status: ms > 0 ? 'listening' : 'not_started',
	}).catch((e) => console.warn('Failed to save progress:', e));
}

// Start/stop auto-save based on play state
audioEngine.on('play', () => startAutoSave());
audioEngine.on('pause', () => stopAutoSave());
audioEngine.on('ended', () => stopAutoSave());

// Save on page close
if (typeof window !== 'undefined') {
	window.addEventListener('beforeunload', () => persistProgress());
}

// -- Actions --

export async function loadAndPlay(
	bookId: number,
	bookTitle: string,
	author: string | null,
	coverPath: string | null,
	chapters: Chapter[],
	resumeMs?: number,
	speed?: number,
) {
	audioEngine.loadBook(chapters);

	// Check for saved progress if no explicit resume position
	if (resumeMs === undefined) {
		try {
			const saved = await getProgress(bookId);
			if (saved && saved.position_ms > 0) {
				resumeMs = saved.position_ms;
				speed = speed ?? saved.playback_speed;
			}
		} catch (e) {
			console.warn('Failed to load progress:', e);
		}
	}

	// Apply speed: saved per-book > explicit > default from settings
	const effectiveSpeed = speed ?? get(settingsStore).default_speed;
	audioEngine.setSpeed(effectiveSpeed);

	playerState.set({
		bookId,
		bookTitle,
		author,
		coverPath,
		chapters,
		currentChapterIndex: 0,
		isPlaying: false,
		currentTimeMs: resumeMs ?? 0,
		totalDurationMs: audioEngine.getTotalDurationMs(),
		playbackSpeed: effectiveSpeed,
		volume: audioEngine.volume,
		isLoaded: true,
		silenceSkip: audioEngine.silenceSkipEnabled,
	});

	audioEngine.play(resumeMs);
}

export function playChapter(index: number) {
	const state = get(playerState);
	if (!state.isLoaded || index < 0 || index >= state.chapters.length) return;
	const chapter = state.chapters[index];
	audioEngine.seekTo(chapter.start_ms);
	if (audioEngine.paused) {
		audioEngine.play();
	}
}

export function toggle() {
	audioEngine.toggle();
}

export function seekTo(ms: number) {
	audioEngine.seekTo(ms);
}

export function seekRelative(deltaMs: number) {
	audioEngine.seekRelative(deltaMs);
}

export function nextChapter() {
	audioEngine.nextChapter();
}

export function prevChapter() {
	audioEngine.prevChapter();
}

export function setSpeed(rate: number) {
	audioEngine.setSpeed(rate);
	playerState.update((s) => ({ ...s, playbackSpeed: audioEngine.speed }));
}

export function adjustSpeed(delta: number) {
	audioEngine.adjustSpeed(delta);
	playerState.update((s) => ({ ...s, playbackSpeed: audioEngine.speed }));
}

export function setVolume(level: number) {
	audioEngine.setVolume(level);
	playerState.update((s) => ({ ...s, volume: audioEngine.volume }));
}

export function toggleSilenceSkip() {
	const next = !audioEngine.silenceSkipEnabled;
	audioEngine.setSilenceSkip(next);
	playerState.update((s) => ({ ...s, silenceSkip: next }));
}

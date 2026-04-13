import { assetUrl, analyzeSilence, type SilenceRegion } from '$lib/api/tauri';
import type { Chapter } from '$lib/types/book';

export interface PlaylistEntry {
	chapter: Chapter;
	assetSrc: string;
	bookOffsetMs: number;
}

export type AudioEngineEvents = {
	timeupdate: { currentTimeMs: number; globalTimeMs: number };
	chapterchange: { index: number };
	ended: void;
	play: void;
	pause: void;
	loadedmetadata: { durationMs: number };
	error: { message: string };
};

type Listener<T> = (data: T) => void;

class AudioEngine {
	private audio: HTMLAudioElement;
	private playlist: PlaylistEntry[] = [];
	private currentIndex = 0;
	private currentSrc = '';
	private _volume = 1;
	private _speed = 1;
	private listeners: { [K in keyof AudioEngineEvents]?: Listener<AudioEngineEvents[K]>[] } = {};

	// Silence skip — precomputed on Rust side, applied during playback
	private _silenceSkipEnabled = false;
	private _silenceThreshold = -50;
	private _silenceMinGapMs = 700;
	// Maps file_path -> array of silence regions (in ms from file start)
	private silenceMaps = new Map<string, SilenceRegion[]>();
	private silenceCheckTimer: ReturnType<typeof setInterval> | null = null;

	constructor() {
		this.audio = new Audio();
		this.audio.preload = 'auto';

		this.audio.addEventListener('timeupdate', () => {
			const entry = this.playlist[this.currentIndex];
			if (!entry) return;
			const globalMs = entry.bookOffsetMs + this.audio.currentTime * 1000;
			this.emit('timeupdate', { currentTimeMs: this.audio.currentTime * 1000, globalTimeMs: globalMs });
			this.checkChapterBoundary(globalMs);
		});

		this.audio.addEventListener('ended', () => {
			if (this.currentIndex < this.playlist.length - 1) {
				this.advanceTo(this.currentIndex + 1);
			} else {
				this.emit('ended', undefined);
			}
		});

		this.audio.addEventListener('play', () => {
			this.emit('play', undefined);
			if (this._silenceSkipEnabled) this.startSilenceCheck();
		});

		this.audio.addEventListener('pause', () => {
			this.emit('pause', undefined);
			this.stopSilenceCheck();
		});

		this.audio.addEventListener('loadedmetadata', () => {
			this.emit('loadedmetadata', { durationMs: this.audio.duration * 1000 });
		});

		this.audio.addEventListener('error', () => {
			this.emit('error', { message: this.audio.error?.message ?? 'Unknown audio error' });
		});
	}

	// ── Playback ──

	loadBook(chapters: Chapter[]) {
		this.audio.pause();
		this.currentSrc = '';
		this.playlist = [];
		this.currentIndex = 0;
		this.silenceMaps.clear();
		for (const ch of chapters) {
			this.playlist.push({
				chapter: ch,
				assetSrc: assetUrl(ch.file_path),
				bookOffsetMs: ch.start_ms,
			});
		}
	}

	async play(seekMs?: number) {
		if (this.playlist.length === 0) return;

		if (seekMs !== undefined) {
			for (let i = 0; i < this.playlist.length; i++) {
				const e = this.playlist[i];
				if (seekMs >= e.chapter.start_ms && seekMs < e.chapter.end_ms) {
					this.currentIndex = i;
					break;
				}
			}
		}

		const entry = this.playlist[this.currentIndex];
		this.loadSource(entry.assetSrc);

		if (seekMs !== undefined) {
			this.audio.currentTime = (seekMs - entry.bookOffsetMs) / 1000;
		}

		this.audio.playbackRate = this._speed;
		this.audio.volume = this._volume;

		try { await this.audio.play(); }
		catch (e) { this.emit('error', { message: String(e) }); }

		// Precompute silence map for current file in background
		if (this._silenceSkipEnabled) {
			this.ensureSilenceMap(entry.chapter.file_path);
		}
	}

	pause() { this.audio.pause(); }

	toggle() {
		if (this.audio.paused) {
			this.audio.playbackRate = this._speed;
			this.audio.volume = this._volume;
			this.audio.play().catch((e) => this.emit('error', { message: String(e) }));
		} else {
			this.audio.pause();
		}
	}

	seekTo(globalMs: number) {
		globalMs = Math.max(0, globalMs);
		const wasPlaying = !this.audio.paused;

		for (let i = 0; i < this.playlist.length; i++) {
			const entry = this.playlist[i];
			if (globalMs >= entry.chapter.start_ms && globalMs < entry.chapter.end_ms) {
				const needsSwitch = this.currentSrc !== entry.assetSrc;
				this.currentIndex = i;
				if (needsSwitch) this.loadSource(entry.assetSrc);
				this.audio.currentTime = (globalMs - entry.bookOffsetMs) / 1000;
				if (wasPlaying && needsSwitch) {
					this.audio.playbackRate = this._speed;
					this.audio.play();
				}
				this.emit('chapterchange', { index: i });
				return;
			}
		}
	}

	seekRelative(deltaMs: number) { this.seekTo(this.getGlobalTimeMs() + deltaMs); }

	nextChapter() {
		if (this.currentIndex < this.playlist.length - 1) this.advanceTo(this.currentIndex + 1);
	}

	prevChapter() {
		const entry = this.playlist[this.currentIndex];
		if (entry) {
			const localMs = this.audio.currentTime * 1000;
			const chapterLocalStart = entry.chapter.start_ms - entry.bookOffsetMs;
			if (localMs - chapterLocalStart > 3000 || this.currentIndex === 0) {
				this.seekTo(entry.chapter.start_ms);
				return;
			}
		}
		if (this.currentIndex > 0) this.advanceTo(this.currentIndex - 1);
	}

	setSpeed(rate: number) {
		this._speed = Math.round(Math.max(0.5, Math.min(3, rate)) * 100) / 100;
		this.audio.playbackRate = this._speed;
	}

	adjustSpeed(delta: number) { this.setSpeed(this._speed + delta); }

	setVolume(level: number) {
		this._volume = Math.max(0, Math.min(1, level));
		this.audio.volume = this._volume;
	}

	// ── Silence skip (precomputed) ──

	get silenceSkipEnabled() { return this._silenceSkipEnabled; }
	get silenceThreshold() { return this._silenceThreshold; }
	get silenceMinGapMs() { return this._silenceMinGapMs; }

	setSilenceSkip(enabled: boolean) {
		this._silenceSkipEnabled = enabled;
		if (enabled) {
			// Analyze current file
			const entry = this.playlist[this.currentIndex];
			if (entry) this.ensureSilenceMap(entry.chapter.file_path);
			if (!this.audio.paused) this.startSilenceCheck();
		} else {
			this.stopSilenceCheck();
		}
	}

	setSilenceThreshold(dbfs: number) {
		this._silenceThreshold = Math.max(-80, Math.min(-10, dbfs));
		// Re-analyze with new threshold
		this.silenceMaps.clear();
		const entry = this.playlist[this.currentIndex];
		if (entry) this.ensureSilenceMap(entry.chapter.file_path);
	}

	setSilenceMinGap(ms: number) {
		this._silenceMinGapMs = Math.max(100, Math.min(5000, ms));
		this.silenceMaps.clear();
		const entry = this.playlist[this.currentIndex];
		if (entry) this.ensureSilenceMap(entry.chapter.file_path);
	}

	/** Precompute silence map for a file on the Rust side */
	private async ensureSilenceMap(filePath: string) {
		if (this.silenceMaps.has(filePath)) return;
		// Mark as in-progress to prevent duplicate analysis
		this.silenceMaps.set(filePath, []);
		try {
			console.log(`[silence] Analyzing: ${filePath.split('/').pop()}`);
			const regions = await analyzeSilence(filePath, this._silenceThreshold, this._silenceMinGapMs);
			this.silenceMaps.set(filePath, regions);
			console.log(`[silence] Found ${regions.length} silence regions`);
		} catch (e) {
			console.warn(`[silence] Analysis failed:`, e);
			this.silenceMaps.delete(filePath);
		}
	}

	/** Check if current playback position is in a silence region and skip past it */
	private startSilenceCheck() {
		this.stopSilenceCheck();
		this.silenceCheckTimer = setInterval(() => {
			if (!this._silenceSkipEnabled || this.audio.paused) return;

			const entry = this.playlist[this.currentIndex];
			if (!entry) return;

			const regions = this.silenceMaps.get(entry.chapter.file_path);
			if (!regions || regions.length === 0) return;

			const currentMs = this.audio.currentTime * 1000;

			// Find if we're inside a silence region
			for (const region of regions) {
				if (currentMs >= region.start_ms && currentMs < region.end_ms) {
					// Skip to end of silence region
					const skipToMs = region.end_ms / 1000;
					console.log(`[silence] Skipping ${((region.end_ms - currentMs) / 1000).toFixed(1)}s of silence`);
					this.audio.currentTime = skipToMs;
					return;
				}
			}
		}, 200); // Check 5 times per second — plenty fast, no CPU waste
	}

	private stopSilenceCheck() {
		if (this.silenceCheckTimer !== null) {
			clearInterval(this.silenceCheckTimer);
			this.silenceCheckTimer = null;
		}
	}

	// ── Getters ──

	get speed() { return this._speed; }
	get volume() { return this._volume; }
	get paused() { return this.audio.paused; }
	get currentChapterIndex() { return this.currentIndex; }

	getGlobalTimeMs(): number {
		const entry = this.playlist[this.currentIndex];
		if (!entry) return 0;
		return entry.bookOffsetMs + this.audio.currentTime * 1000;
	}

	getTotalDurationMs(): number {
		if (this.playlist.length === 0) return 0;
		return this.playlist[this.playlist.length - 1].chapter.end_ms;
	}

	// ── Internal ──

	private loadSource(src: string) {
		if (this.currentSrc === src) return;
		this.audio.src = src;
		this.currentSrc = src;
	}

	private advanceTo(index: number) {
		const wasPlaying = !this.audio.paused;
		this.currentIndex = index;
		const entry = this.playlist[index];
		this.loadSource(entry.assetSrc);
		const localStart = (entry.chapter.start_ms - entry.bookOffsetMs) / 1000;
		if (localStart > 0) this.audio.currentTime = localStart;
		this.audio.playbackRate = this._speed;
		this.audio.volume = this._volume;
		if (wasPlaying) this.audio.play();
		this.emit('chapterchange', { index });

		// Pre-analyze next file for silence
		if (this._silenceSkipEnabled) {
			this.ensureSilenceMap(entry.chapter.file_path);
		}
	}

	private checkChapterBoundary(globalMs: number) {
		const entry = this.playlist[this.currentIndex];
		if (!entry) return;
		if (globalMs >= entry.chapter.end_ms && this.currentIndex < this.playlist.length - 1) {
			const next = this.playlist[this.currentIndex + 1];
			if (next && next.chapter.file_path === entry.chapter.file_path) {
				this.currentIndex++;
				this.emit('chapterchange', { index: this.currentIndex });
			}
		}
	}

	// ── Events ──

	on<K extends keyof AudioEngineEvents>(event: K, fn: Listener<AudioEngineEvents[K]>) {
		if (!this.listeners[event]) (this.listeners as any)[event] = [];
		(this.listeners[event] as any[]).push(fn);
	}

	off<K extends keyof AudioEngineEvents>(event: K, fn: Listener<AudioEngineEvents[K]>) {
		const arr = this.listeners[event] as any[] | undefined;
		if (arr) { const i = arr.indexOf(fn); if (i !== -1) arr.splice(i, 1); }
	}

	private emit<K extends keyof AudioEngineEvents>(event: K, data: AudioEngineEvents[K]) {
		const arr = this.listeners[event] as any[] | undefined;
		if (arr) for (const fn of arr) fn(data);
	}
}

export const audioEngine = new AudioEngine();

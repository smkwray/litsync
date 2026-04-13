import { writable, get } from 'svelte/store';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getSettings, saveSettings } from '$lib/api/tauri';
import type { AppSettings, AppTheme } from '$lib/types/book';
import { audioEngine } from '$lib/utils/audio';

const defaults: AppSettings = {
	theme: 'dark',
	default_speed: 1.0,
	skip_short_sec: 10,
	skip_long_sec: 30,
	auto_rewind_enabled: true,
	auto_rewind_short_pause_sec: 30,
	auto_rewind_short_rewind_sec: 5,
	auto_rewind_long_pause_sec: 300,
	auto_rewind_long_rewind_sec: 30,
	sleep_timer_default_min: 30,
	silence_skip_threshold: -50,
	silence_skip_min_gap_ms: 700,
};

export const settingsStore = writable<AppSettings>(defaults);

let loaded = false;
const WINDOW_BACKGROUND: Record<AppTheme, string> = {
	dark: '#0d0f14',
	light: '#f6f1e7'
};

function applyAudioSettings(settings: AppSettings) {
	audioEngine.setSilenceThreshold(settings.silence_skip_threshold);
	audioEngine.setSilenceMinGap(settings.silence_skip_min_gap_ms);
}

function applyTheme(theme: AppTheme) {
	if (typeof document !== 'undefined') {
		document.documentElement.dataset.theme = theme;
		document.documentElement.style.colorScheme = theme;
	}

	if (typeof window !== 'undefined') {
		const currentWindow = getCurrentWindow();
		currentWindow.setTheme(theme).catch(() => {});
		currentWindow.setBackgroundColor(WINDOW_BACKGROUND[theme]).catch(() => {});
	}
}

function applySettings(settings: AppSettings) {
	applyAudioSettings(settings);
	applyTheme(settings.theme);
}

if (typeof document !== 'undefined') {
	applyTheme(defaults.theme);
}

export async function loadSettings() {
	try {
		const s = await getSettings();
		settingsStore.set(s);
		applySettings(s);
		loaded = true;
	} catch {
		settingsStore.set(defaults);
		applySettings(defaults);
	}
}

export async function updateSettings(partial: Partial<AppSettings>) {
	const current = get(settingsStore);
	const updated = { ...current, ...partial };
	settingsStore.set(updated);
	applySettings(updated);
	try {
		await saveSettings(updated);
	} catch (e) {
		console.warn('Failed to save settings:', e);
	}
}

export function ensureSettingsLoaded() {
	if (!loaded) loadSettings();
}

import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

// Tauri expects a fixed port and disables the host check for the dev server
const maybeProcess = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process;
const isTauri = maybeProcess?.env?.TAURI_ENV_PLATFORM !== undefined;

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		host: isTauri ? false : undefined
	}
});

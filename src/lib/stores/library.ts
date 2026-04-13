import { writable } from 'svelte/store';
import { getBooks as fetchBooks } from '$lib/api/tauri';
import type { Book } from '$lib/types/book';

/** Persists across page navigations so the grid is immediately available on back-nav. */
export const booksStore = writable<Book[]>([]);

let loaded = false;

export async function loadBooks() {
	try {
		const books = await fetchBooks();
		booksStore.set(books);
		loaded = true;
	} catch (e) {
		console.error('Failed to load books:', e);
	}
}

export async function reloadBooks() {
	loaded = false;
	return loadBooks();
}

export function ensureBooksLoaded() {
	if (!loaded) loadBooks();
}

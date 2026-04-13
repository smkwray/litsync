export const prerender = false;

export function load({ params }: { params: { id: string } }) {
	return { id: Number(params.id) };
}

import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, cookies }) => {
	const body = await request.json();
	const dcauth = cookies.get('sckk-dc-auth');
	if (dcauth) {
		const mama = await fetch('http://localhost:3000/potlek/upload', {
			mode: 'no-cors',
			headers: {
				cookie: cookies.get('sckk-dc-auth') as string,
				'Content-Type': 'application/json'
			},
			method: 'post',
			body: JSON.stringify({
				img: body.img,
				createdAt: body.createdAt,
				selected: body.selected
			})
		});
		if (mama.ok) {
			return new Response(await mama.text());
		}
	}
	return new Response(body);
};

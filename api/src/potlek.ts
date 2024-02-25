import express from 'express';
import { oauth } from './lib/discord.ts';
import { getTag } from './lib/google.ts';
import { prisma } from './lib/prisma.ts';

export const router = express.Router();

router.get('/', async (req, res) => {
	if (!req.headers.cookie) return res.sendStatus(404);
	let cookie: string | undefined = undefined;
	for (const kuki of JSON.parse(req.headers.cookie)) {
		if (kuki.name === 'sckk-dc-auth') {
			cookie = kuki.value;
		}
	}
	if (!cookie) return res.sendStatus(404);
	const user = await oauth.getUser(cookie);
	if (user) {
		const doksi = await getTag(user.id);
		if (doksi) {
			const cuccok = await prisma.data.findMany({
				where: {
					owner: doksi.name as string
				},
				select: {
					date: true,
					id: true,
					status: true
				}
			});
			if (cuccok[0]) {
				res.send(cuccok);
			}
		} else {
			res.sendStatus(401);
		}
	} else {
		res.sendStatus(404);
	}
});

router.post('/upload', async (req, res) => {
	if (!req.headers.cookie) return res.sendStatus(404);
	const body = await req.body;
	if (!req.headers.cookie) return res.sendStatus(404);
	if (!body) return res.sendStatus(400);
	const user = await oauth.getUser(req.headers.cookie);
	if (user) {
		const doksi = await getTag(user.id);
		if (doksi) {
			const kep = await prisma.data.create({
				data: {
					owner: doksi.name as string,
					kep: body.img,
					type: body.selected
				}
			});
			if (kep) {
				res.send(kep.id.toString());
			} else {
				res.sendStatus(400);
			}
		} else {
			res.sendStatus(401);
		}
	} else {
		res.sendStatus(404);
	}
});

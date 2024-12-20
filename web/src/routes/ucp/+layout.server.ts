import { type Redirect, redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";
import { apiUrl } from "$lib/api";

export const load = (async ({ cookies, request, url }) => {
	if (!cookies.get("auth_token")) {
		throw redirect(302, `${apiUrl}/auth?path=${url.pathname}`);
	}
	try {
		const aha = await fetch(`${apiUrl}/ucp`, {
			headers: {
				cookie: cookies.get("auth_token")!,
			},
		});
		if (aha.status === 404 || aha.status === 406) {
			throw redirect(
				302,
				`${apiUrl}/auth?path=${url.pathname}`,
			);
		}
		if (aha.status === 403) {
			return {
				noaccess: true,
			};
		}
		if (aha.ok) {
			const jeson = await aha.json();
			return {
				layout: jeson,
				api: apiUrl,
				country: process.env.NODE_ENV === "development"
					? "HU"
					: (request.headers.get("cf-ipcountry") as string),
				auth: cookies.get("auth_token")!,
				offset: process.env.SUMMER_TIMEZONE === "true"
					? -60 * 60 * 1000 * 2
					: -60 * 60 * 1000,
				music: jeson.admin
					? (cookies.get("play_music") === "true" ? true : false)
					: false,
				agent: request.headers.get("user-agent") as string,
				maintenance: cookies.get("maintenance")
					? jeson.admin ? cookies.get("maintenance") : false
					: false,
			};
		}
	} catch (err) {
		if ((err as Redirect).status) {
			throw redirect(
				(err as Redirect).status,
				(err as Redirect).location,
			);
		}
		return {
			error: "Weboldal API szerverét nem sikerült elérni",
		};
	}
}) satisfies LayoutServerLoad;

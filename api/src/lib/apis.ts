import 'dotenv/config';

export function getApiUrl(api: 'erik' | 'patrik') {
	if (api === 'erik') {
		return 'https://app.sckk.hu';
	}
	if (api === 'patrik') {
		return 'http://api.scms.hanrickio.com:5002';
	}
}

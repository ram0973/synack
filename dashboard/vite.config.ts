import { fileURLToPath } from 'node:url';
import adapter from '@sveltejs/adapter-static';
import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

const apiTarget = process.env.VITE_API_TARGET ?? 'http://127.0.0.1:3000';

const proxy = {
	'/api': apiTarget,
	'/media': apiTarget
};

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			adapter: adapter({ fallback: 'index.html' }),
			alias: {
				$lib: fileURLToPath(new URL('./src/lib', import.meta.url))
			}
		})
	],
	server: { proxy },
	preview: { proxy }
});

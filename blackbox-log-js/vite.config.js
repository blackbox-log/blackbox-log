import { viteStaticCopy as staticCopy } from 'vite-plugin-static-copy';
import fs from 'fs';
import { defineConfig } from 'vite';

export default defineConfig({
	build: {
		lib: {
			entry: ['src/main.ts', 'src/slim.ts'],
			name: 'BlackboxLog',
			formats: ['es', 'cjs'],
			fileName: (format, entry) => {
				const ext = format == 'cjs' ? 'cjs' : 'js';
				return `${entry}.${ext}`;
			},
		},
	},
	plugins: [
		{
			name: 'inline-wasm',
			async transform(_, id) {
				if (id.endsWith('.wasm?inline')) {
					const path = id.replace('?inline', '');
					const wasm = await fs.promises.readFile(path, { encoding: 'base64' });
					return `export default '${wasm}'`;
				}
			},
		},
		staticCopy({
			targets: [
				{
					src: 'src/blackbox-log.wasm',
					dest: '',
				},
			],
			watch: {
				reloadPageOnChange: true,
			},
		}),
	],
});

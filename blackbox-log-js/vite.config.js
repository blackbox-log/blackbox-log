import fs from 'fs';
import { defineConfig } from 'vite';
import { viteStaticCopy as staticCopy } from 'vite-plugin-static-copy';

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
		sourcemap: true,
	},
	plugins: [
		{
			name: 'inline-wasm',
			async transform(_, id) {
				if (id.endsWith('.wasm?inline')) {
					const path = id.replace('?inline', '');
					const wasm = await fs.promises.readFile(path, { encoding: 'base64' });
					return {
						code: `export default '${wasm}'`,
						map: { mappings: '' },
					};
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

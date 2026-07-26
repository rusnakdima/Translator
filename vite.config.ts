import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 5174,
    strictPort: true
  },
  resolve: {
    alias: {
      '$lib': path.resolve('./src/lib'),
      '@tauri-front/shared': path.resolve('../tauri-front-shared/projects/shared/dist')
    }
  }
});

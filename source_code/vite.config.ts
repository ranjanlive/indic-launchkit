import { defineConfig } from 'vite';
export default defineConfig({ clearScreen: false, server: { port: 1420, strictPort: true, fs: { allow: ['.', '../assets/fonts'] }, watch: { ignored: ['**/target/**', '**/src-tauri/**'] } }, build: { target: 'es2022' } });

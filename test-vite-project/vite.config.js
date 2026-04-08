import { defineConfig } from 'vite';
import euis from 'vite-plugin-euis';

export default defineConfig({
  plugins: [
    euis({
      minify: false,
      sourceMaps: true,
      treeShaking: false,
    }),
  ],
});

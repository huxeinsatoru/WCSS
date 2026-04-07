import { defineConfig } from 'vite';
import wcss from 'vite-plugin-wcss';

export default defineConfig({
  plugins: [
    wcss({
      minify: false,
      sourceMaps: true,
      treeShaking: false,
    }),
  ],
});

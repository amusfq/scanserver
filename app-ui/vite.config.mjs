import { defineConfig } from "vite";
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import path from "path";
import VueI18nPlugin from '@intlify/unplugin-vue-i18n/vite'
import packageJson from './package.json' with { type: 'json' };

// https://vitejs.dev/config/
export default defineConfig({
  base: '',
  server: {
    proxy: {
      '/api': 'http://localhost:8080',
      '/api-docs': 'http://localhost:8080',
    }
  },
  plugins: [
    vue(),
    tailwindcss(),
    VueI18nPlugin({
      include: [path.resolve(import.meta.dirname, './src/locales/**')],
      compositionOnly: false
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  define: {
    __PACKAGE_VERSION__: JSON.stringify(packageJson.version)
  },
});

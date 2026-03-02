import { defineConfig } from 'vite'
export default defineConfig({
  preview: {
    port: 58080,
    host: 'localhost',
    proxy: {
      '/api': { target: 'http://localhost:3141', changeOrigin: true },
      '/events': { target: 'http://localhost:3141', changeOrigin: true },
    },
  },
})

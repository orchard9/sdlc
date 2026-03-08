import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            // Markdown processing pipeline (checked first — react-markdown would otherwise match the react group)
            if (/react-markdown|remark|rehype|unified|mdast|micromark|hast|unist|vfile|bail|trough|property-information|comma-separated|space-separated|decode-named|character-entities|ccount|escape-string|devlop|stringify-entities/.test(id)) {
              return 'vendor-markdown'
            }
            // Mermaid diagramming
            if (/mermaid|dagre|d3|khroma|cytoscape|elkjs|dompurify|lodash|stylis/.test(id)) {
              return 'vendor-mermaid'
            }
            // React core ecosystem (checked last so react-markdown is already handled above)
            if (/\/react\/|\/react-dom\/|\/react-router|\/scheduler\//.test(id)) {
              return 'vendor-react'
            }
          }
        },
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3141',
        changeOrigin: true,
      },
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    exclude: ['e2e/**', 'node_modules/**'],
  },
})

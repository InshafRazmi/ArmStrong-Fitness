import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig(({ mode }) => ({
  plugins: [react()],
  define: { 'import.meta.env.VITE_DESKTOP_ONLY': JSON.stringify(mode === 'desktop' ? 'true' : 'false') },
  server: { host: '0.0.0.0', port: 4173 },
}))

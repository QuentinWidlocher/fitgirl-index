import { type Config } from "tailwindcss"

export default {
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  theme: {
    extend: {
      fontFamily: {
        sans: ['DM Sans Variable']
      }
    },
  },
  plugins: [],
} satisfies Config

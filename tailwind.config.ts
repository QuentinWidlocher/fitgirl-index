import { type Config } from "tailwindcss"

export default {
  content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
  theme: {
    extend: {
      animation: {
        'bounce-once': 'bounce-once 1s 3'
      },
      keyframes: {
        'bounce-once': {
          '0%, 100%': { transform: 'translateY(0)', 'animation-timing-function': 'cubic-bezier(0, 0, 0.2, 1)' },
          '50%': { transform: 'translateY(-25%)', 'animation-timing-function': 'cubic-bezier(0.8, 0, 1, 1)' },
        }
      },
      fontFamily: {
        sans: ['DM Sans Variable']
      }
    },
  },
  plugins: [],
} satisfies Config

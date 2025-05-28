/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {},
  },
  safelist: [
    // DaisyUI base colors
    'bg-base-100',
    'bg-base-200',
    'bg-base-300',
    'bg-neutral',
    'border-base-100',
    'border-base-200',
    'border-base-300',
    'text-base-100',
    'text-base-200',
    'text-base-300',
    'text-base-content',
  ]
};

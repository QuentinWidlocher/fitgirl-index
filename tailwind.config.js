/** @type {import('tailwindcss').Config} */
module.exports = {
	content: [
		'./src/**/*.html',
		'./src/**/*.rs',
	],
	theme: {
		extend: {
			fontFamily: {
				sans: ['DM Sans Variable']
			}
		},
	},
	plugins: [],
}

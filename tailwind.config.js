module.exports = {
  purge: ['./*.html', './src/**/*.{svelte,js,ts,css}'],
  darkMode: false, // or 'media' or 'class'
  theme: {
    container: {
      center: true,
      padding: {
        DEFAULT: "1rem",
        sm: "1.5rem",
        lg: "2rem"
      }
    },
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [],
}

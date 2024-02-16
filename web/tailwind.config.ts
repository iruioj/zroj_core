import color from "tailwindcss/colors";
import type { Config } from "tailwindcss";
import plugin from "tailwindcss/plugin";

const config: Partial<Config> = {
  darkMode: "class",
  content: [
    "./components/**/*.{js,vue,ts}",
    "./layouts/**/*.vue",
    "./pages/**/*.vue",
    "./plugins/**/*.{js,ts}",
    "./nuxt.config.{js,ts}",
    "./app.vue",
  ],
  theme: {
    screens: {
      sm: "480px",
      md: "768px",
      lg: "976px",
      xl: "1440px",
    },
    colors: {
      brand: "rgb(var(--theme-brand) / <alpha-value>)",
      "brand-secondary": "rgb(var(--theme-brand-sec) / <alpha-value>)",
      red: color.red,
      white: color.white,
      slate: color.slate,
      black: color.black,
      green: color.green,
      cool: color.gray,
      back: "rgb(var(--theme-bg) / <alpha-value>)",
      primary: "rgb(var(--theme-pri) / <alpha-value>)",
      secondary: "rgb(var(--theme-sec) / <alpha-value>)",
    },
    fontFamily: {
      sans: [
        "-apple-system",
        "BlinkMacSystemFont",
        "Segoe UI",
        "Roboto",
        "Oxygen",
        "Ubuntu",
        "Cantarell",
        "Fira Sans",
        "Droid Sans",
        "Helvetica Neue",
        "sans-serif",
      ],
      serif: [
        "Times New Roman",
        "Times",
        "Roboto",
        "Oxygen",
        "Ubuntu",
        "Cantarell",
        "Fira Sans",
        "Droid Sans",
        "Helvetica Neue",
        "serif",
      ],
      mono: [
        "Menlo",
        "Monaco",
        "Consolas",
        "andale mono",
        "ubuntu mono",
        "courier new",
        "monospace",
      ],
    },
    extend: {
      spacing: {
        "128": "32rem",
        "144": "36rem",
      },
      borderRadius: {
        "4xl": "2rem",
      },
    },
  },
  plugins: [
    plugin(function ({ addBase, theme }) {
      const back = "rgb(var(--theme-bg))";
      const front = "rgb(var(--theme-pri))";
      addBase({
        input: {
          "background-color": back,
        },
        "input:autofill": {
          "-webkit-box-shadow": "0 0 0 30px " + back + " inset !important",
          "-webkit-text-fill-color": `${front} !important`,
        },
        html: {
          color: "rgb(var(--theme-pri))",
          "background-color": back,
        },
        // '@media (prefers-color-scheme: dark)': {
        //   'html': {
        //     'background-color':
        //   },
        // },
      });
    }),
  ],
};
export default config;

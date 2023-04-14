import color from "tailwindcss/colors";
import { Config } from "tailwindcss";
import plugin from "tailwindcss/plugin";

const config: Partial<Config> = {
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
      back: "rgb(var(--theme-bg) / <alpha-value>)",
      // primary: 'rgb(var(--theme-pri) / <alpha-value>)',
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
      addBase({
        "input:autofill": {
          "-webkit-box-shadow":
            "0 0 0 30px " + theme("colors.white") + " inset !important",
          "-webkit-text-fill-color": `${theme("colors.brand")} !important`,
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

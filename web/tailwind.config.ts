import color from "tailwindcss/colors";
import { Config } from "tailwindcss";

const config: Partial<Config> = {
  theme: {
    screens: {
      sm: "480px",
      md: "768px",
      lg: "976px",
      xl: "1440px",
    },
    colors: {
      brand: "#8c0000",
      "brand-light": "#fee2e2",
      "brand-dark": "#560000",
      red: color.red,
      white: color.white,
      slate: color.slate,
      black: color.black,
      // ...color,
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
};
export default config;

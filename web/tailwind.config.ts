import color from 'tailwindcss/colors'
import { Config } from 'tailwindcss'

const config: Partial<Config> = {
  theme: {
    screens: {
      sm: '480px',
      md: '768px',
      lg: '976px',
      xl: '1440px',
    },
    colors: {
      'brand': '#8c0000',
      'brand-light': '#fee2e2',
      red: color.red,
      white: color.white,
      slate: color.slate,
      black: color.black,
      // ...color,
    },
    // fontFamily: {
    //   sans: ['Graphik', 'sans-serif'],
    //   serif: ['Merriweather', 'serif'],
    // },
    extend: {
      spacing: {
        '128': '32rem',
        '144': '36rem',
      },
      borderRadius: {
        '4xl': '2rem',
      }
    }
  }
}
export default config;
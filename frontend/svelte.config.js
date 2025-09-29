import sveltePreprocess from 'svelte-preprocess';
import tailwindcss from 'tailwindcss';
import autoprefixer from 'autoprefixer';

const config = {
  preprocess: sveltePreprocess({
    sourceMap: true,
    postcss: {
      plugins: [tailwindcss, autoprefixer],
    },
  }),
};

export default config;
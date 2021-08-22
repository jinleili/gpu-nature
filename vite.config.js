import vue from '@vitejs/plugin-vue'

export default {
  // config doc： https://vitejs.dev/config/#async-config
  entry: 'index.html',
  base: './',
  root: 'web/static/',
  plugins: [vue()],
};

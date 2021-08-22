module.exports = {
  root: true,
  parser: 'vue-eslint-parser',
  parserOptions: {
    sourceType: 'module',
  },
  env: {
    browser: true,
    node: true,
    es6: true,
  },
  extends: [
    'plugin:vue/vue3-recommended',
    // 'plugin:prettier/recommended', // Enables eslint-plugin-prettier and eslint-config-prettier. This will display prettier errors as ESLint errors. Make sure this is always the last configuration in the extends array.
  ],
  rules: {
    // 规则文档：https://vuejs.github.io/eslint-plugin-vue/rules/html-closing-bracket-newline.html
    'no-console': 'off',
    'space-before-function-paren': ['error', 'never'],
    'brace-style': ['error', '1tbs', { allowSingleLine: true }],
    'vue/html-closing-bracket-newline': ['error', {
      'singleline': 'never',
      'multiline': 'never'
    }],
    'vue/no-multi-spaces': ['error', {
      'ignoreProperties': false
    }],
    'vue/html-self-closing': ['error', {
      'html': {
        'void': 'never',
        'normal': 'always',
        'component': 'always'
      },
      'svg': 'always',
      'math': 'always'
    }],
    'vue/max-attributes-per-line': [
      'error',
      {
        singleline: 5,
        multiline: 4,
      },
    ],
    'vue/html-indent': [
      'error',
      2,
      {
        attribute: 1,
        baseIndent: 1,
        closeBracket: 0,
        alignAttributesVertically: true,
        ignores: [],
      },
    ],
  },
}

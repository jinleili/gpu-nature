const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./web/index.js"
  },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    static: {
      directory: dist,
    },
    compress: true,
    port: 8080,
  },
  module: {
    // https://webpack.js.org/concepts/loaders/
    rules: [
      {
        test: /\.sass$/,
        use: [{ loader: 'sass-loader' }]
      },
      {
        test: /\.wasm$/,
        type: 'webassembly/sync',
      }
    ],
  },
  experiments: {
    syncWebAssembly: true
  },
  // avoid wasm file size limit, default only 244KB
  performance: {
    hints: false
  },
  plugins: [
    new CopyPlugin({
      patterns: [
        path.resolve(__dirname, "web/static/dist")
      ]
    }),

    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),
  ]
};

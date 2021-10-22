const path = require('path')

const CopyPlugin = require('copy-webpack-plugin')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

module.exports = {
  mode: 'production',
  plugins: [
    new CopyPlugin([{
      from: 'res'
    }]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '.')
    })
  ],
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /(node_modules|html-entities)/,
        // include: /(node_modules|@holochain)/,
        use: ["remove-hashbag-loader"]
      }
    ]
  },
  resolveLoader: {
    alias: {
      "remove-hashbag-loader": path.join(__dirname, "./res/loaders/remove-hashbag-loader.js")
    }
  }
}

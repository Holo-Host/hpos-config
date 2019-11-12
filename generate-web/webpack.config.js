const path = require('path')
const webpack = require('webpack')

const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const HtmlWebpackPlugin = require('html-webpack-plugin')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'target', 'webpack'),
    filename: 'index.js'
  },
  plugins: [
    new MiniCssExtractPlugin({
      filename: 'style.css'
    }),
    new HtmlWebpackPlugin({
      template: 'index.html'
    }),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '.')
    }),
    // https://caniuse.com/#feat=textencoder
    new webpack.ProvidePlugin({
      TextDecoder: ['text-encoding', 'TextDecoder'],
      TextEncoder: ['text-encoding', 'TextEncoder']
    })
  ],
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [MiniCssExtractPlugin.loader, 'css-loader']
      }
    ]
  },
  mode: 'production'
}

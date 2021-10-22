const path = require('path')
const CopyPlugin = require('copy-webpack-plugin')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')
const HtmlWebpackPlugin = require('html-webpack-plugin');

module.exports = {
  mode: 'production',
  plugins: [
    new CopyPlugin([{
      from: 'res'
    }]),
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, '.')
    }),
    new HtmlWebpackPlugin({
      filename: 'index.html',
      template: './res/index.html' 
   })
  ],
  experiments: {
    asyncWebAssembly: true
  },
  entry: './src/index.js',
  output: {
      path: path.resolve(__dirname, 'dist'),
      filename: 'bundle.js'
  },
  devServer: {
      contentBase: './dist'
  },
  module: {
      rules: [
          {
              test: /\.js$/, //using regex to tell babel exactly what files to transcompile
              exclude: /node_modules/, // files to be ignored
              use: {
                  loader: 'babel-loader' // specify the loader
              } 
          }
      ]
  },
  resolve: {
    fallback: {
      "path": false,
      "crypto": false,
      "crypto-browserify": require.resolve('crypto-browserify')
    } 
  }
}

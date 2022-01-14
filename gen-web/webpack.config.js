const path = require('path')
const CopyPlugin = require('copy-webpack-plugin')
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin')
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');

if (!process.env.MEMBRANE_PROOF_SERVICE_URL) {
  throw new Error('Please define MEMBRANE_PROOF_SERVICE_URL environment variable. Typically https://membrane-proof-service.holo.host')
}

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
    }),
    new webpack.DefinePlugin({
      'process.env':{
        'MEMBRANE_PROOF_SERVICE_URL': JSON.stringify(process.env.MEMBRANE_PROOF_SERVICE_URL),
      }
    })
  ],
  experiments: {
    syncWebAssembly: true
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

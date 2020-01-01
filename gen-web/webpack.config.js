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
  ]
}

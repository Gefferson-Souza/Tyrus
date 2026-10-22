'use strict';

const path = require('node:path');

const isBareSpecifier = (request) => !request.startsWith('.') && !path.isAbsolute(request);

module.exports = {
  mode: 'production',
  target: 'node',
  entry: { main: path.resolve(__dirname, 'src/main.ts') },
  output: {
    path: path.resolve(__dirname, 'dist/bundle'),
    filename: '[name].js',
    clean: true,
  },
  resolve: { extensions: ['.ts', '.js'] },
  module: {
    rules: [
      {
        test: /\.ts$/,
        loader: 'ts-loader',
        exclude: /node_modules/,
        options: { configFile: path.resolve(__dirname, 'tsconfig.json'), transpileOnly: false },
      },
    ],
  },
  externals: [
    ({ request }, callback) =>
      request !== undefined && isBareSpecifier(request)
        ? callback(null, `commonjs ${request}`)
        : callback(),
  ],
  optimization: { minimize: false, moduleIds: 'deterministic', chunkIds: 'deterministic' },
  node: { __dirname: false, __filename: false },
};

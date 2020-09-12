const prod = require('./webpack.prod.js');
const merge = require('webpack-merge');

module.exports = merge(prod, {
  output: {
    libraryTarget: 'commonjs2',
    libraryExport: 'default',
    library: 'converse',
  },
});

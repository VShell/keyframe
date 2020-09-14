const fs = require('fs');
const path = require('path');
const util = require('util');
const checker = require('license-checker');

const readFile = util.promisify(fs.readFile);

const uiRoot = path.resolve(__dirname, '..');
const keyframeRoot = path.resolve(__dirname, '../..');

module.exports = async () => {
  const licenses = await util.promisify(checker.init)({
    start: uiRoot,
    production: true,
  });
  const converseLicenses = JSON.parse(await readFile(path.resolve(uiRoot, 'converse/dist/licenses.json')));
  Object.assign(licenses, converseLicenses);
  const licenseData = {};
  for (const [name, license] of Object.entries(licenses)) {
    licenseData[name] = {
      license: license.licenses,
      licenseText: license.licenseFile ? await readFile(license.licenseFile, 'utf8') : undefined,
    };
  }

  const backendLicenses = require('./backend-licenses.json');
  const backendLicenseData = {};
  for (const [name, license] of Object.entries(backendLicenses)) {
    backendLicenseData[name] = {
      license: license.license,
      licenseText: await readFile(path.resolve(keyframeRoot, license.licenseFile), 'utf8'),
    };
  }

  return {
    code: 'module.exports = '+JSON.stringify({licenses: licenseData, backendLicenses: backendLicenseData})+';',
  };
};

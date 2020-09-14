# This file has been generated by node2nix 1.8.0. Do not edit!

{nodeEnv, fetchurl, fetchgit, globalBuildInputs ? []}:

let
  sources = {
    "abbrev-1.1.1" = {
      name = "abbrev";
      packageName = "abbrev";
      version = "1.1.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/abbrev/-/abbrev-1.1.1.tgz";
        sha512 = "nne9/IiQ/hzIhY6pdDnbBtz7DjPTKrY00P/zvPSm5pOFkl6xuGrGnXn/VtTNNfNtAfZ9/1RtehkszU9qcTii0Q==";
      };
    };
    "ansi-styles-3.2.1" = {
      name = "ansi-styles";
      packageName = "ansi-styles";
      version = "3.2.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/ansi-styles/-/ansi-styles-3.2.1.tgz";
        sha512 = "VT0ZI6kZRdTh8YyJw3SMbYm/u+NqfsAxEpWO0Pf9sq8/e94WxxOpPKx9FR1FlyCtOVDNOQ+8ntlqFxiRc+r5qA==";
      };
    };
    "array-find-index-1.0.2" = {
      name = "array-find-index";
      packageName = "array-find-index";
      version = "1.0.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/array-find-index/-/array-find-index-1.0.2.tgz";
        sha1 = "df010aa1287e164bbda6f9723b0a96a1ec4187a1";
      };
    };
    "asap-2.0.6" = {
      name = "asap";
      packageName = "asap";
      version = "2.0.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/asap/-/asap-2.0.6.tgz";
        sha1 = "e50347611d7e690943208bbdafebcbc2fb866d46";
      };
    };
    "balanced-match-1.0.0" = {
      name = "balanced-match";
      packageName = "balanced-match";
      version = "1.0.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/balanced-match/-/balanced-match-1.0.0.tgz";
        sha1 = "89b4d199ab2bee49de164ea02b89ce462d71b767";
      };
    };
    "brace-expansion-1.1.11" = {
      name = "brace-expansion";
      packageName = "brace-expansion";
      version = "1.1.11";
      src = fetchurl {
        url = "https://registry.npmjs.org/brace-expansion/-/brace-expansion-1.1.11.tgz";
        sha512 = "iCuPHDFgrHX7H2vEI/5xpz07zSHB00TpugqhmYtVmMO6518mCuRMoOYFldEBl0g187ufozdaHgWKcYFb61qGiA==";
      };
    };
    "chalk-2.4.2" = {
      name = "chalk";
      packageName = "chalk";
      version = "2.4.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/chalk/-/chalk-2.4.2.tgz";
        sha512 = "Mti+f9lpJNcwF4tWV8/OrTTtF1gZi+f8FqlyAdouralcFWFQWF2+NgCHShjkCb+IFBLq9buZwE1xckQU4peSuQ==";
      };
    };
    "color-convert-1.9.3" = {
      name = "color-convert";
      packageName = "color-convert";
      version = "1.9.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/color-convert/-/color-convert-1.9.3.tgz";
        sha512 = "QfAUtd+vFdAtFQcC8CCyYt1fYWxSqAiK2cSD6zDB8N3cpsEBAvRxp9zOGg6G/SHHJYAT88/az/IuDGALsNVbGg==";
      };
    };
    "color-name-1.1.3" = {
      name = "color-name";
      packageName = "color-name";
      version = "1.1.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/color-name/-/color-name-1.1.3.tgz";
        sha1 = "a7d0558bd89c42f795dd42328f740831ca53bc25";
      };
    };
    "concat-map-0.0.1" = {
      name = "concat-map";
      packageName = "concat-map";
      version = "0.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/concat-map/-/concat-map-0.0.1.tgz";
        sha1 = "d8a96bd77fd68df7793a73036a3ba0d5405d477b";
      };
    };
    "debug-3.2.6" = {
      name = "debug";
      packageName = "debug";
      version = "3.2.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/debug/-/debug-3.2.6.tgz";
        sha512 = "mel+jf7nrtEl5Pn1Qx46zARXKDpBbvzezse7p7LqINmdoIk8PYP5SySaxEmYv6TZ0JyEKA1hsCId6DIhgITtWQ==";
      };
    };
    "debuglog-1.0.1" = {
      name = "debuglog";
      packageName = "debuglog";
      version = "1.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/debuglog/-/debuglog-1.0.1.tgz";
        sha1 = "aa24ffb9ac3df9a2351837cfb2d279360cd78492";
      };
    };
    "dezalgo-1.0.3" = {
      name = "dezalgo";
      packageName = "dezalgo";
      version = "1.0.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/dezalgo/-/dezalgo-1.0.3.tgz";
        sha1 = "7f742de066fc748bc8db820569dddce49bf0d456";
      };
    };
    "escape-string-regexp-1.0.5" = {
      name = "escape-string-regexp";
      packageName = "escape-string-regexp";
      version = "1.0.5";
      src = fetchurl {
        url = "https://registry.npmjs.org/escape-string-regexp/-/escape-string-regexp-1.0.5.tgz";
        sha1 = "1b61c0562190a8dff6ae3bb2cf0200ca130b86d4";
      };
    };
    "fs.realpath-1.0.0" = {
      name = "fs.realpath";
      packageName = "fs.realpath";
      version = "1.0.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/fs.realpath/-/fs.realpath-1.0.0.tgz";
        sha1 = "1504ad2523158caa40db4a2787cb01411994ea4f";
      };
    };
    "glob-7.1.6" = {
      name = "glob";
      packageName = "glob";
      version = "7.1.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/glob/-/glob-7.1.6.tgz";
        sha512 = "LwaxwyZ72Lk7vZINtNNrywX0ZuLyStrdDtabefZKAY5ZGJhVtgdznluResxNmPitE0SAO+O26sWTHeKSI2wMBA==";
      };
    };
    "graceful-fs-4.2.4" = {
      name = "graceful-fs";
      packageName = "graceful-fs";
      version = "4.2.4";
      src = fetchurl {
        url = "https://registry.npmjs.org/graceful-fs/-/graceful-fs-4.2.4.tgz";
        sha512 = "WjKPNJF79dtJAVniUlGGWHYGz2jWxT6VhN/4m1NdkbZ2nOsEF+cI1Edgql5zCRhs/VsQYRvrXctxktVXZUkixw==";
      };
    };
    "has-flag-3.0.0" = {
      name = "has-flag";
      packageName = "has-flag";
      version = "3.0.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/has-flag/-/has-flag-3.0.0.tgz";
        sha1 = "b5d454dc2199ae225699f3467e5a07f3b955bafd";
      };
    };
    "hosted-git-info-2.8.8" = {
      name = "hosted-git-info";
      packageName = "hosted-git-info";
      version = "2.8.8";
      src = fetchurl {
        url = "https://registry.npmjs.org/hosted-git-info/-/hosted-git-info-2.8.8.tgz";
        sha512 = "f/wzC2QaWBs7t9IYqB4T3sR1xviIViXJRJTWBlx2Gf3g0Xi5vI7Yy4koXQ1c9OYDGHN9sBy1DQ2AB8fqZBWhUg==";
      };
    };
    "inflight-1.0.6" = {
      name = "inflight";
      packageName = "inflight";
      version = "1.0.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/inflight/-/inflight-1.0.6.tgz";
        sha1 = "49bd6331d7d02d0c09bc910a1075ba8165b56df9";
      };
    };
    "inherits-2.0.4" = {
      name = "inherits";
      packageName = "inherits";
      version = "2.0.4";
      src = fetchurl {
        url = "https://registry.npmjs.org/inherits/-/inherits-2.0.4.tgz";
        sha512 = "k/vGaX4/Yla3WzyMCvTQOXYeIHvqOKtnqBduzTHpzpQZzAskKMhZ2K+EnBiSM9zGSoIFeMpXKxa4dYeZIQqewQ==";
      };
    };
    "json-parse-even-better-errors-2.3.1" = {
      name = "json-parse-even-better-errors";
      packageName = "json-parse-even-better-errors";
      version = "2.3.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/json-parse-even-better-errors/-/json-parse-even-better-errors-2.3.1.tgz";
        sha512 = "xyFwyhro/JEof6Ghe2iz2NcXoj2sloNsWr/XsERDK/oiPCfaNhl5ONfp+jQdAZRQQ0IJWNzH9zIZF7li91kh2w==";
      };
    };
    "minimatch-3.0.4" = {
      name = "minimatch";
      packageName = "minimatch";
      version = "3.0.4";
      src = fetchurl {
        url = "https://registry.npmjs.org/minimatch/-/minimatch-3.0.4.tgz";
        sha512 = "yJHVQEhyqPLUTgt9B83PXu6W3rx4MvvHvSUvToogpwoGDOUQ+yDrR0HRot+yOCdCO7u4hX3pWft6kWBBcqh0UA==";
      };
    };
    "minimist-1.2.5" = {
      name = "minimist";
      packageName = "minimist";
      version = "1.2.5";
      src = fetchurl {
        url = "https://registry.npmjs.org/minimist/-/minimist-1.2.5.tgz";
        sha512 = "FM9nNUYrRBAELZQT3xeZQ7fmMOBg6nWNmJKTcgsJeaLstP/UODVpGsr5OhXhhXg6f+qtJ8uiZ+PUxkDWcgIXLw==";
      };
    };
    "mkdirp-0.5.5" = {
      name = "mkdirp";
      packageName = "mkdirp";
      version = "0.5.5";
      src = fetchurl {
        url = "https://registry.npmjs.org/mkdirp/-/mkdirp-0.5.5.tgz";
        sha512 = "NKmAlESf6jMGym1++R0Ra7wvhV+wFW63FaSOFPwRahvea0gMUcGUhVeAg/0BC0wiv9ih5NYPB1Wn1UEI1/L+xQ==";
      };
    };
    "ms-2.1.2" = {
      name = "ms";
      packageName = "ms";
      version = "2.1.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/ms/-/ms-2.1.2.tgz";
        sha512 = "sGkPx+VjMtmA6MX27oA4FBFELFCZZ4S4XqeGOXCv68tT+jb3vk/RyaKWP0PTKyWtmLSM0b+adUTEvbs1PEaH2w==";
      };
    };
    "nopt-4.0.3" = {
      name = "nopt";
      packageName = "nopt";
      version = "4.0.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/nopt/-/nopt-4.0.3.tgz";
        sha512 = "CvaGwVMztSMJLOeXPrez7fyfObdZqNUK1cPAEzLHrTybIua9pMdmmPR5YwtfNftIOMv3DPUhFaxsZMNTQO20Kg==";
      };
    };
    "normalize-package-data-2.5.0" = {
      name = "normalize-package-data";
      packageName = "normalize-package-data";
      version = "2.5.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/normalize-package-data/-/normalize-package-data-2.5.0.tgz";
        sha512 = "/5CMN3T0R4XTj4DcGaexo+roZSdSFW/0AOOTROrjxzCG1wrWXEsGbRKevjlIL+ZDE4sZlJr5ED4YW0yqmkK+eA==";
      };
    };
    "npm-normalize-package-bin-1.0.1" = {
      name = "npm-normalize-package-bin";
      packageName = "npm-normalize-package-bin";
      version = "1.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/npm-normalize-package-bin/-/npm-normalize-package-bin-1.0.1.tgz";
        sha512 = "EPfafl6JL5/rU+ot6P3gRSCpPDW5VmIzX959Ob1+ySFUuuYHWHekXpwdUZcKP5C+DS4GEtdJluwBjnsNDl+fSA==";
      };
    };
    "once-1.4.0" = {
      name = "once";
      packageName = "once";
      version = "1.4.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/once/-/once-1.4.0.tgz";
        sha1 = "583b1aa775961d4b113ac17d9c50baef9dd76bd1";
      };
    };
    "os-homedir-1.0.2" = {
      name = "os-homedir";
      packageName = "os-homedir";
      version = "1.0.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/os-homedir/-/os-homedir-1.0.2.tgz";
        sha1 = "ffbc4988336e0e833de0c168c7ef152121aa7fb3";
      };
    };
    "os-tmpdir-1.0.2" = {
      name = "os-tmpdir";
      packageName = "os-tmpdir";
      version = "1.0.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/os-tmpdir/-/os-tmpdir-1.0.2.tgz";
        sha1 = "bbe67406c79aa85c5cfec766fe5734555dfa1274";
      };
    };
    "osenv-0.1.5" = {
      name = "osenv";
      packageName = "osenv";
      version = "0.1.5";
      src = fetchurl {
        url = "https://registry.npmjs.org/osenv/-/osenv-0.1.5.tgz";
        sha512 = "0CWcCECdMVc2Rw3U5w9ZjqX6ga6ubk1xDVKxtBQPK7wis/0F2r9T6k4ydGYhecl7YUBxBVxhL5oisPsNxAPe2g==";
      };
    };
    "path-is-absolute-1.0.1" = {
      name = "path-is-absolute";
      packageName = "path-is-absolute";
      version = "1.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/path-is-absolute/-/path-is-absolute-1.0.1.tgz";
        sha1 = "174b9268735534ffbc7ace6bf53a5a9e1b5c5f5f";
      };
    };
    "path-parse-1.0.6" = {
      name = "path-parse";
      packageName = "path-parse";
      version = "1.0.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/path-parse/-/path-parse-1.0.6.tgz";
        sha512 = "GSmOT2EbHrINBf9SR7CDELwlJ8AENk3Qn7OikK4nFYAu3Ote2+JYNVvkpAEQm3/TLNEJFD/xZJjzyxg3KBWOzw==";
      };
    };
    "read-installed-4.0.3" = {
      name = "read-installed";
      packageName = "read-installed";
      version = "4.0.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/read-installed/-/read-installed-4.0.3.tgz";
        sha1 = "ff9b8b67f187d1e4c29b9feb31f6b223acd19067";
      };
    };
    "read-package-json-2.1.2" = {
      name = "read-package-json";
      packageName = "read-package-json";
      version = "2.1.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/read-package-json/-/read-package-json-2.1.2.tgz";
        sha512 = "D1KmuLQr6ZSJS0tW8hf3WGpRlwszJOXZ3E8Yd/DNRaM5d+1wVRZdHlpGBLAuovjr28LbWvjpWkBHMxpRGGjzNA==";
      };
    };
    "readdir-scoped-modules-1.1.0" = {
      name = "readdir-scoped-modules";
      packageName = "readdir-scoped-modules";
      version = "1.1.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/readdir-scoped-modules/-/readdir-scoped-modules-1.1.0.tgz";
        sha512 = "asaikDeqAQg7JifRsZn1NJZXo9E+VwlyCfbkZhwyISinqk5zNS6266HS5kah6P0SaQKGF6SkNnZVHUzHFYxYDw==";
      };
    };
    "resolve-1.17.0" = {
      name = "resolve";
      packageName = "resolve";
      version = "1.17.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/resolve/-/resolve-1.17.0.tgz";
        sha512 = "ic+7JYiV8Vi2yzQGFWOkiZD5Z9z7O2Zhm9XMaTxdJExKasieFCr+yXZ/WmXsckHiKl12ar0y6XiXDx3m4RHn1w==";
      };
    };
    "semver-5.7.1" = {
      name = "semver";
      packageName = "semver";
      version = "5.7.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/semver/-/semver-5.7.1.tgz";
        sha512 = "sauaDf/PZdVgrLTNYHRtpXa1iRiKcaebiKQ1BJdpQlWH2lCvexQdX55snPFyK7QzpudqbCI0qXFfOasHdyNDGQ==";
      };
    };
    "slide-1.1.6" = {
      name = "slide";
      packageName = "slide";
      version = "1.1.6";
      src = fetchurl {
        url = "https://registry.npmjs.org/slide/-/slide-1.1.6.tgz";
        sha1 = "56eb027d65b4d2dce6cb2e2d32c4d4afc9e1d707";
      };
    };
    "spdx-compare-1.0.0" = {
      name = "spdx-compare";
      packageName = "spdx-compare";
      version = "1.0.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-compare/-/spdx-compare-1.0.0.tgz";
        sha512 = "C1mDZOX0hnu0ep9dfmuoi03+eOdDoz2yvK79RxbcrVEG1NO1Ph35yW102DHWKN4pk80nwCgeMmSY5L25VE4D9A==";
      };
    };
    "spdx-correct-3.1.1" = {
      name = "spdx-correct";
      packageName = "spdx-correct";
      version = "3.1.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-correct/-/spdx-correct-3.1.1.tgz";
        sha512 = "cOYcUWwhCuHCXi49RhFRCyJEK3iPj1Ziz9DpViV3tbZOwXD49QzIN3MpOLJNxh2qwq2lJJZaKMVw9qNi4jTC0w==";
      };
    };
    "spdx-exceptions-2.3.0" = {
      name = "spdx-exceptions";
      packageName = "spdx-exceptions";
      version = "2.3.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-exceptions/-/spdx-exceptions-2.3.0.tgz";
        sha512 = "/tTrYOC7PPI1nUAgx34hUpqXuyJG+DTHJTnIULG4rDygi4xu/tfgmq1e1cIRwRzwZgo4NLySi+ricLkZkw4i5A==";
      };
    };
    "spdx-expression-parse-3.0.1" = {
      name = "spdx-expression-parse";
      packageName = "spdx-expression-parse";
      version = "3.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-expression-parse/-/spdx-expression-parse-3.0.1.tgz";
        sha512 = "cbqHunsQWnJNE6KhVSMsMeH5H/L9EpymbzqTQ3uLwNCLZ1Q481oWaofqH7nO6V07xlXwY6PhQdQ2IedWx/ZK4Q==";
      };
    };
    "spdx-license-ids-3.0.5" = {
      name = "spdx-license-ids";
      packageName = "spdx-license-ids";
      version = "3.0.5";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-license-ids/-/spdx-license-ids-3.0.5.tgz";
        sha512 = "J+FWzZoynJEXGphVIS+XEh3kFSjZX/1i9gFBaWQcB+/tmpe2qUsSBABpcxqxnAxFdiUFEgAX1bjYGQvIZmoz9Q==";
      };
    };
    "spdx-ranges-2.1.1" = {
      name = "spdx-ranges";
      packageName = "spdx-ranges";
      version = "2.1.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-ranges/-/spdx-ranges-2.1.1.tgz";
        sha512 = "mcdpQFV7UDAgLpXEE/jOMqvK4LBoO0uTQg0uvXUewmEFhpiZx5yJSZITHB8w1ZahKdhfZqP5GPEOKLyEq5p8XA==";
      };
    };
    "spdx-satisfies-4.0.1" = {
      name = "spdx-satisfies";
      packageName = "spdx-satisfies";
      version = "4.0.1";
      src = fetchurl {
        url = "https://registry.npmjs.org/spdx-satisfies/-/spdx-satisfies-4.0.1.tgz";
        sha512 = "WVzZ/cXAzoNmjCWiEluEA3BjHp5tiUmmhn9MK+X0tBbR9sOqtC6UQwmgCNrAIZvNlMuBUYAaHYfb2oqlF9SwKA==";
      };
    };
    "supports-color-5.5.0" = {
      name = "supports-color";
      packageName = "supports-color";
      version = "5.5.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/supports-color/-/supports-color-5.5.0.tgz";
        sha512 = "QjVjwdXIt408MIiAqCX4oUKsgU2EqAGzs2Ppkm4aQYbjm+ZEWEcW4SfFNTr4uMNZma0ey4f5lgLrkB0aX0QMow==";
      };
    };
    "treeify-1.1.0" = {
      name = "treeify";
      packageName = "treeify";
      version = "1.1.0";
      src = fetchurl {
        url = "https://registry.npmjs.org/treeify/-/treeify-1.1.0.tgz";
        sha512 = "1m4RA7xVAJrSGrrXGs0L3YTwyvBs2S8PbRHaLZAkFw7JR8oIFwYtysxlBZhYIa7xSyiYJKZ3iGrrk55cGA3i9A==";
      };
    };
    "util-extend-1.0.3" = {
      name = "util-extend";
      packageName = "util-extend";
      version = "1.0.3";
      src = fetchurl {
        url = "https://registry.npmjs.org/util-extend/-/util-extend-1.0.3.tgz";
        sha1 = "a7c216d267545169637b3b6edc6ca9119e2ff93f";
      };
    };
    "validate-npm-package-license-3.0.4" = {
      name = "validate-npm-package-license";
      packageName = "validate-npm-package-license";
      version = "3.0.4";
      src = fetchurl {
        url = "https://registry.npmjs.org/validate-npm-package-license/-/validate-npm-package-license-3.0.4.tgz";
        sha512 = "DpKm2Ui/xN7/HQKCtpZxoRWBhZ9Z0kqtygG8XCgNQ8ZlDnxuQmWhj566j8fN4Cu3/JmbhsDo7fcAJq4s9h27Ew==";
      };
    };
    "wrappy-1.0.2" = {
      name = "wrappy";
      packageName = "wrappy";
      version = "1.0.2";
      src = fetchurl {
        url = "https://registry.npmjs.org/wrappy/-/wrappy-1.0.2.tgz";
        sha1 = "b5243d8f3ec1aa35f1364605bc0d1036e30ab69f";
      };
    };
  };
in
{
  "license-checker-25.0.1" = nodeEnv.buildNodePackage {
    name = "license-checker";
    packageName = "license-checker";
    version = "25.0.1";
    src = fetchurl {
      url = "https://registry.npmjs.org/license-checker/-/license-checker-25.0.1.tgz";
      sha512 = "mET5AIwl7MR2IAKYYoVBBpV0OnkKQ1xGj2IMMeEFIs42QAkEVjRtFZGWmQ28WeU7MP779iAgOaOy93Mn44mn6g==";
    };
    dependencies = [
      sources."abbrev-1.1.1"
      sources."ansi-styles-3.2.1"
      sources."array-find-index-1.0.2"
      sources."asap-2.0.6"
      sources."balanced-match-1.0.0"
      sources."brace-expansion-1.1.11"
      sources."chalk-2.4.2"
      sources."color-convert-1.9.3"
      sources."color-name-1.1.3"
      sources."concat-map-0.0.1"
      sources."debug-3.2.6"
      sources."debuglog-1.0.1"
      sources."dezalgo-1.0.3"
      sources."escape-string-regexp-1.0.5"
      sources."fs.realpath-1.0.0"
      sources."glob-7.1.6"
      sources."graceful-fs-4.2.4"
      sources."has-flag-3.0.0"
      sources."hosted-git-info-2.8.8"
      sources."inflight-1.0.6"
      sources."inherits-2.0.4"
      sources."json-parse-even-better-errors-2.3.1"
      sources."minimatch-3.0.4"
      sources."minimist-1.2.5"
      sources."mkdirp-0.5.5"
      sources."ms-2.1.2"
      sources."nopt-4.0.3"
      sources."normalize-package-data-2.5.0"
      sources."npm-normalize-package-bin-1.0.1"
      sources."once-1.4.0"
      sources."os-homedir-1.0.2"
      sources."os-tmpdir-1.0.2"
      sources."osenv-0.1.5"
      sources."path-is-absolute-1.0.1"
      sources."path-parse-1.0.6"
      sources."read-installed-4.0.3"
      sources."read-package-json-2.1.2"
      sources."readdir-scoped-modules-1.1.0"
      sources."resolve-1.17.0"
      sources."semver-5.7.1"
      sources."slide-1.1.6"
      sources."spdx-compare-1.0.0"
      sources."spdx-correct-3.1.1"
      sources."spdx-exceptions-2.3.0"
      sources."spdx-expression-parse-3.0.1"
      sources."spdx-license-ids-3.0.5"
      sources."spdx-ranges-2.1.1"
      sources."spdx-satisfies-4.0.1"
      sources."supports-color-5.5.0"
      sources."treeify-1.1.0"
      sources."util-extend-1.0.3"
      sources."validate-npm-package-license-3.0.4"
      sources."wrappy-1.0.2"
    ];
    buildInputs = globalBuildInputs;
    meta = {
      description = "Check license info for a package";
      homepage = "https://github.com/davglass/license-checker#readme";
      license = "BSD-3-Clause";
    };
    production = true;
    bypassCache = true;
    reconstructLock = true;
  };
}
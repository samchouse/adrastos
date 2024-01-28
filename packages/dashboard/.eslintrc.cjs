module.exports = {
  extends: ['../../.eslintrc'],
  env: { browser: true, es2020: true, node: true },
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: ['./tsconfig.json'],
  },
};

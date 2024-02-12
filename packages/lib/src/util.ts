export const merge = (...strings: (string | undefined)[]) =>
  strings.filter(Boolean).join('').trim();

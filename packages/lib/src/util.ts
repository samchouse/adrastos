export function merge(...strings: (string | undefined)[]) {
  return strings.filter(Boolean).join('').trim();
}

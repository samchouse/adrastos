export function NotFound() {
  return (
    <div className="flex h-full flex-col items-center justify-center">
      <h1 className="font-bold text-4xl">Oops!</h1>
      <h2 className="font-light text-lg text-muted-foreground">
        This page does't seem to exist
      </h2>
    </div>
  );
}

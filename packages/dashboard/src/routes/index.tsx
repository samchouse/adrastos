import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/')({
  component: IndexComponent,
});

export function IndexComponent() {
  return (
    <div className="mx-5 flex h-full flex-col items-center justify-center gap-y-1 text-center">
      <h1 className="text-4xl font-bold text-blue-500">Adrastos</h1>
      <h2 className="text-muted-foreground text-lg font-light">
        A killer Backend-as-a-Service (BaaS) written in Rust
      </h2>
    </div>
  );
}

import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_layout/')({
  component: IndexComponent,
});

function IndexComponent() {
  return (
    <div className="mx-5 flex h-full flex-col items-center justify-center gap-y-1 text-center">
      <h1 className="text-4xl font-bold text-blue-500">Adrastos</h1>
      <h2 className="text-lg font-light text-muted-foreground">
        A killer Backend-as-a-Service (BaaS) written in Rust
      </h2>
    </div>
  );
}

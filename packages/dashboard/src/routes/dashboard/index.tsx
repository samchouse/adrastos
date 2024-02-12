import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/dashboard/')({
  component: IndexComponent,
});

function IndexComponent() {
  return <div></div>;
}

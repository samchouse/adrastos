import { createLazyFileRoute } from '@tanstack/react-router';

export const Route = createLazyFileRoute('/dashboard/')({
  component: IndexComponent,
});

function IndexComponent() {
  return <div></div>;
}

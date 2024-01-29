import { createLazyFileRoute } from '@tanstack/react-router';

export const Route = createLazyFileRoute('/dashboard/auth')({
  component: () => <div>Hello /dashboard/auth!</div>,
});

import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/dashboard/auth')({
  component: () => <div>Hello /dashboard/auth!</div>,
});

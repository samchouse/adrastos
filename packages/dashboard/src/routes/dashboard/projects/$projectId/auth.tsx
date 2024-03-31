import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/dashboard/projects/$projectId/auth')({
  component: () => <div>Hello /dashboard/projects/$projectId/auth!</div>,
});

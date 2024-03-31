import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/dashboard/projects/$projectId/storage')({
  component: () => <div>Hello /dashboard/projects/$projectId/storage!</div>,
});

import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/dashboard/teams/$teamId/settings')({
  component: () => <div>Hello /dashboard/teams/settings!</div>,
});

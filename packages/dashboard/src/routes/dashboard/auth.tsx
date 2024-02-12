import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/dashboard/auth')({
  component: AuthComponent,
});

function AuthComponent() {
  return <div></div>;
}

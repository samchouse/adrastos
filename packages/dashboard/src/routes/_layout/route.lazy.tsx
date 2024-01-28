import { createLazyFileRoute, Link, Outlet } from '@tanstack/react-router';

import { Button } from '~/components/ui';
import { useMeQuery } from '~/hooks';

export const Route = createLazyFileRoute('/_layout')({
  component: RouteComponent,
});

function RouteComponent() {
  const { data: user } = useMeQuery();

  return (
    <>
      <header className="bg-background relative z-10 flex w-screen justify-between border-b px-4 py-3">
        <Link to="/">
          <img
            src="/logo.svg"
            alt="logo"
            width={40}
            height={40}
            className="mr-2"
          />
        </Link>

        {user ? (
          <Button asChild>
            <Link to="/dashboard">Dashboard</Link>
          </Button>
        ) : (
          <div className="space-x-3">
            <Button asChild variant="outline">
              <Link to="/login">Login</Link>
            </Button>
            <Button asChild>
              <Link to="/signup">Signup</Link>
            </Button>
          </div>
        )}
      </header>

      <main className="bg-background h-full">
        <Outlet />
      </main>
    </>
  );
}

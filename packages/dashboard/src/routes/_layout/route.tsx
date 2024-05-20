import { createFileRoute, Outlet } from '@tanstack/react-router';

import { Navbar } from '~/components';
import { meQueryOptions } from '~/hooks';

export const Route = createFileRoute('/_layout')({
  component: RouteComponent,
  loader: async ({ context: { queryClient, client } }) =>
    await queryClient
      .ensureQueryData(meQueryOptions(client))
      .catch(() => undefined),
});

function RouteComponent() {
  const user = Route.useLoaderData();

  return (
    <>
      <Navbar user={user} />

      <main className="h-full overflow-y-auto bg-background">
        <Outlet />
      </main>
    </>
  );
}

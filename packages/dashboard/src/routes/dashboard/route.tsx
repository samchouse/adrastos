import { Outlet, createFileRoute } from '@tanstack/react-router';

import { meQueryOptions, teamsQueryOptions } from '~/hooks';

export const Route = createFileRoute('/dashboard')({
  component: RouteComponent,
  loader: async ({ context: { queryClient, client } }) => ({
    me: await queryClient.ensureQueryData(meQueryOptions(client)),
    teams: await queryClient.ensureQueryData(teamsQueryOptions(client)),
  }),
});

function RouteComponent() {
  return <Outlet />;
}

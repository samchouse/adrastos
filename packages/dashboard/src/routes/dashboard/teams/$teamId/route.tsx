import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute, notFound, Outlet } from '@tanstack/react-router';

import { Navbar } from '~/components';
import {
  meQueryOptions,
  projectsQueryOptions,
  teamsQueryOptions,
} from '~/hooks';

export const Route = createFileRoute('/dashboard/teams/$teamId')({
  component: RouteComponent,
  loader: async ({ params: { teamId }, context: { queryClient, client } }) =>
    await queryClient.ensureQueryData(projectsQueryOptions(client, teamId)),
  beforeLoad: async ({
    context: { queryClient, client },
    params: { teamId },
  }) => {
    const teams = await queryClient.ensureQueryData(teamsQueryOptions(client));
    if (!teams.find((team) => team.id === teamId)) throw notFound();
  },
});

function RouteComponent() {
  const { teamId } = Route.useParams();
  const { client } = Route.useRouteContext();

  const [{ data: user }, { data: teams }] = useSuspenseQueries({
    queries: [meQueryOptions(client), teamsQueryOptions(client)],
  });

  return (
    <>
      <Navbar user={user} teamId={teamId} teams={teams} />

      <main className="h-full overflow-y-auto bg-background">
        <Outlet />
      </main>
    </>
  );
}

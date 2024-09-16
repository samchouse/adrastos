import { useSuspenseQueries } from '@tanstack/react-query';
import { Outlet, createFileRoute, notFound } from '@tanstack/react-router';

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
    // eslint-disable-next-line @typescript-eslint/only-throw-error
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
      <Navbar user={user} teams={teams} teamId={teamId} />

      <main className="h-full overflow-y-auto bg-background">
        <Outlet />
      </main>
    </>
  );
}

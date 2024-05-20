import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute, notFound, Outlet } from '@tanstack/react-router';

import { Navbar, NotFound } from '~/components';
import {
  meQueryOptions,
  projectQueryOptions,
  teamsQueryOptions,
} from '~/hooks';

export const Route = createFileRoute('/dashboard/projects/$projectId')({
  component: RouteComponent,
  notFoundComponent: NotFound,
  beforeLoad: async ({
    context: { client, queryClient },
    params: { projectId },
  }) => {
    const project = await queryClient
      .ensureQueryData(projectQueryOptions(client, projectId))
      .catch(() => undefined);
    if (!project) throw notFound();

    client.setProjectId(projectId, true);
  },
});

function RouteComponent() {
  const { projectId } = Route.useParams();
  const { client } = Route.useRouteContext();

  const [{ data: user }, { data: teams }, { data: project }] =
    useSuspenseQueries({
      queries: [
        meQueryOptions(client),
        teamsQueryOptions(client),
        projectQueryOptions(client, projectId),
      ],
    });

  return (
    <>
      <Navbar
        user={user}
        teamId={project.teamId}
        teams={teams}
        project={project}
      />

      <main className="h-full overflow-y-auto bg-background">
        <Outlet />
      </main>
    </>
  );
}

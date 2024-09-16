import { useSuspenseQueries } from '@tanstack/react-query';
import {
  Link,
  Outlet,
  createFileRoute,
  redirect,
} from '@tanstack/react-router';
import clsx from 'clsx';
import { title } from 'radash';

import { Button } from '~/components';
import { tablesQueryOptions } from '~/hooks';

import { TableSheet } from './-components';

export const Route = createFileRoute('/dashboard/projects/$projectId/tables')({
  component: RouteComponent,
  loader: async ({ context: { client, queryClient } }) =>
    await queryClient.ensureQueryData(tablesQueryOptions(client)),
  beforeLoad: async ({
    location,
    params: { projectId },
    context: { client, queryClient },
  }) => {
    const tables = await queryClient.ensureQueryData(
      tablesQueryOptions(client),
    );

    if (tables.length !== 0 && location.pathname.endsWith('/tables'))
      // eslint-disable-next-line @typescript-eslint/only-throw-error
      throw redirect({
        to: '/dashboard/projects/$projectId/tables/$tableId',
        params: {
          projectId: projectId,
          tableId: tables[0].name,
        },
      });
  },
});

function RouteComponent() {
  const { projectId } = Route.useParams();
  const { client } = Route.useRouteContext();

  const [{ data: tables }] = useSuspenseQueries({
    queries: [tablesQueryOptions(client)],
  });

  return (
    <section className="flex size-full flex-row">
      {tables.length === 0 ? (
        <div className="flex size-full flex-col items-center justify-center space-y-5">
          <h1 className="text-2xl text-muted-foreground">
            No tables have been created
          </h1>
          <TableSheet client={client} tables={tables} className="w-72" />
        </div>
      ) : (
        <>
          <div className="flex h-full w-72 flex-col justify-between border-r p-4">
            <div>
              <h2 className="mb-2 ml-3 font-semibold text-lg">Tables</h2>
              <div className="flex flex-col space-y-1">
                {tables.map((table) => (
                  <Link
                    key={table.id}
                    to="/dashboard/projects/$projectId/tables/$tableId"
                    params={{
                      projectId: projectId,
                      tableId: table.name,
                    }}
                  >
                    {({ isActive }) => (
                      <Button
                        variant="ghost"
                        className={clsx(
                          'w-full justify-start',
                          isActive ? 'bg-muted' : 'hover:bg-muted/50',
                        )}
                      >
                        {title(table.name)}
                      </Button>
                    )}
                  </Link>
                ))}
              </div>
            </div>

            <TableSheet client={client} tables={tables} />
          </div>

          <div className="w-full">
            <Outlet />
          </div>
        </>
      )}
    </section>
  );
}

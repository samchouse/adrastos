import { useSuspenseQueries } from '@tanstack/react-query';
import {
  createFileRoute,
  Link,
  Outlet,
  redirect,
} from '@tanstack/react-router';
import clsx from 'clsx';
import { title } from 'radash';

import { Button } from '~/components';
import { tablesQueryOptions } from '~/hooks';

import { TableSheet } from './-components';

export const Route = createFileRoute('/dashboard/tables')({
  component: RouteComponent,
  loader: async ({ context: { client, queryClient } }) =>
    await queryClient.ensureQueryData(tablesQueryOptions(client)),
  beforeLoad: async ({ location, context: { client, queryClient } }) => {
    const tables = await queryClient.ensureQueryData(
      tablesQueryOptions(client),
    );

    if (tables.length !== 0 && location.pathname === '/dashboard/tables')
      throw redirect({
        to: '/dashboard/tables/$tableId',
        params: { tableId: tables?.[0].name },
      });
  },
});

function RouteComponent() {
  const { client } = Route.useRouteContext();

  const [{ data: tables }] = useSuspenseQueries({
    queries: [tablesQueryOptions(client)],
  });

  return (
    <section className="flex h-full w-full flex-row">
      {tables.length === 0 ? (
        <div className="flex h-full w-full flex-col items-center justify-center space-y-5">
          <h1 className="text-muted-foreground text-2xl">
            No tables have been created
          </h1>
          <TableSheet client={client} className="w-72" />
        </div>
      ) : (
        <>
          <div className="flex h-full w-72 flex-col justify-between border-r p-4">
            <div>
              <h2 className="mb-2 ml-3 text-lg font-semibold">Tables</h2>
              <div className="flex flex-col space-y-1">
                {tables?.map((table) => (
                  <Link
                    key={table.id}
                    to={`/dashboard/tables/$tableId`}
                    params={{ tableId: table.name }}
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

            <TableSheet client={client} />
          </div>

          <div className="w-full">
            <Outlet />
          </div>
        </>
      )}
    </section>
  );
}

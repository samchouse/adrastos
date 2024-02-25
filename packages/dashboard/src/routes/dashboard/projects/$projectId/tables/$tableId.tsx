import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute, notFound } from '@tanstack/react-router';
import {
  ColumnDef,
  createColumnHelper,
  SortingState,
} from '@tanstack/react-table';
import { format } from 'date-fns';
import {
  ChevronDown,
  ChevronRight,
  ChevronUp,
  MoreHorizontal,
} from 'lucide-react';
import { title } from 'radash';
import { useEffect, useMemo, useState } from 'react';

import {
  Badge,
  Button,
  Checkbox,
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  NotFound,
} from '~/components';
import { tableDataQueryOptions, tablesQueryOptions } from '~/hooks';
import { cn } from '~/lib';

import { DataTable, Row, RowSheet, TableSheet } from './-components';

export const Route = createFileRoute(
  '/dashboard/projects/$projectId/tables/$tableId',
)({
  component: TableIdComponent,
  notFoundComponent: NotFound,
  loader: async ({
    context: { client, queryClient },
    params: { tableId },
  }) => ({
    tables: await queryClient.ensureQueryData(tablesQueryOptions(client)),
    data: await queryClient
      .ensureQueryData(
        tableDataQueryOptions<Row, false>(client, tableId, false),
      )
      .catch(() => {
        throw notFound();
      }),
  }),
});

const columnHelper = createColumnHelper<Row>();

function TableIdComponent() {
  const { client } = Route.useRouteContext();
  const { tableId } = Route.useParams();
  const [sorting, setSorting] = useState<SortingState>([
    { id: 'createdAt', desc: true },
  ]);

  const [lastTableId, setLastTableId] = useState(tableId);

  useEffect(() => {
    if (lastTableId !== tableId) {
      setSorting([{ id: 'createdAt', desc: true }]);
      setLastTableId(tableId);
    }
  }, [tableId, lastTableId]);

  const [{ data: tables }, { data }] = useSuspenseQueries({
    queries: [
      tablesQueryOptions(client),
      tableDataQueryOptions<Row, false>(client, tableId, false),
    ],
  });

  const table = useMemo(
    () => tables?.find((t) => t.name === tableId),
    [tables, tableId],
  );

  const columns = useMemo(
    () => [
      columnHelper.display({
        id: 'checkbox',
        meta: {
          style: {
            width: 'min',
          },
        },
        header: ({ table }) => (
          <Checkbox
            checked={
              table.getIsAllPageRowsSelected() ||
              (table.getIsSomePageRowsSelected() && 'indeterminate')
            }
            onCheckedChange={(value) =>
              table.toggleAllPageRowsSelected(!!value)
            }
            aria-label="Select all"
          />
        ),
        cell: ({ row }) => (
          <Checkbox
            checked={row.getIsSelected()}
            onCheckedChange={(value) => row.toggleSelected(!!value)}
            onClick={(e) => e.stopPropagation()}
            aria-label="Select row"
          />
        ),
      }),
      columnHelper.accessor('id', {
        header: 'Id',
        meta: {
          style: {
            width: 'min',
          },
        },
      }) as ColumnDef<Row>,
      ...(table?.fields.map((f) =>
        columnHelper.accessor(f.name, {
          enableSorting: true,
          header: ({ column }) => (
            <Button
              variant="ghost"
              className="group"
              onClick={() =>
                column.toggleSorting(
                  column.getIsSorted() ? column.getIsSorted() === 'asc' : true,
                )
              }
            >
              {title(f.name)}
              {column.getIsSorted() === 'asc' ? (
                <ChevronUp
                  className={cn(
                    'invisible ml-2 h-4 w-4 group-hover:visible',
                    column.getIsSorted() && 'visible',
                  )}
                />
              ) : (
                <ChevronDown
                  className={cn(
                    'invisible ml-2 h-4 w-4 group-hover:visible',
                    column.getIsSorted() && 'visible',
                  )}
                />
              )}
            </Button>
          ),
          cell: ({ getValue, renderValue }) => {
            const value = getValue?.();
            return !value ? (
              <Badge variant="secondary">N/A</Badge>
            ) : (
              <>{renderValue?.() ?? value}</>
            );
          },
        }),
      ) ?? []),
      columnHelper.display({ id: 'no-fields' }),
      columnHelper.accessor('createdAt', {
        enableSorting: true,
        meta: {
          style: {
            width: 'min',
          },
        },
        header: ({ column }) => (
          <Button
            variant="ghost"
            className="group"
            onClick={() =>
              column.toggleSorting(
                column.getIsSorted() ? column.getIsSorted() === 'asc' : true,
              )
            }
          >
            Created At
            {column.getIsSorted() === 'asc' ? (
              <ChevronUp
                className={cn(
                  'invisible ml-2 h-4 w-4 group-hover:visible',
                  column.getIsSorted() && 'visible',
                )}
              />
            ) : (
              <ChevronDown
                className={cn(
                  'invisible ml-2 h-4 w-4 group-hover:visible',
                  column.getIsSorted() && 'visible',
                )}
              />
            )}
          </Button>
        ),
        cell: ({ getValue }) => {
          const value = getValue?.() as Date;
          return (
            <>
              <p className="mb-[3px] leading-none">
                {format(value, 'MM-dd-yyyy')}
              </p>
              <p className="text-muted-foreground leading-none">
                {format(value, 'h:mm:ss a')}
              </p>
            </>
          );
        },
      }) as ColumnDef<Row>,
      columnHelper.accessor('updatedAt', {
        enableSorting: true,
        meta: {
          style: {
            width: 'min',
          },
        },
        header: ({ column }) => (
          <Button
            variant="ghost"
            className="group"
            onClick={() =>
              column.toggleSorting(
                column.getIsSorted() ? column.getIsSorted() === 'asc' : true,
              )
            }
          >
            Updated At
            {column.getIsSorted() === 'asc' ? (
              <ChevronUp
                className={cn(
                  'invisible ml-2 h-4 w-4 group-hover:visible',
                  column.getIsSorted() && 'visible',
                )}
              />
            ) : (
              <ChevronDown
                className={cn(
                  'invisible ml-2 h-4 w-4 group-hover:visible',
                  column.getIsSorted() && 'visible',
                )}
              />
            )}
          </Button>
        ),
        cell: ({ getValue }) => {
          const value = getValue?.() as Date;
          return !value ? (
            <Badge variant="secondary">Never</Badge>
          ) : (
            <>
              <p className="mb-[3px] leading-none">
                {format(value, 'MM-dd-yyyy')}
              </p>
              <p className="text-muted-foreground leading-none">
                {format(value, 'h:mm:ss a')}
              </p>
            </>
          );
        },
      }) as ColumnDef<Row>,
      columnHelper.display({
        id: 'actions',
        meta: {
          style: {
            width: 'min',
            textAlign: 'right',
          },
        },
        header: ({ table }) => (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="h-8 w-8 p-0">
                <MoreHorizontal className="h-4 w-4 text-white" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-[150px]">
              <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {table
                .getAllColumns()
                .filter(
                  (column) =>
                    typeof column.accessorFn !== 'undefined' &&
                    column.getCanHide(),
                )
                .map((column) => (
                  <DropdownMenuCheckboxItem
                    key={column.id}
                    checked={column.getIsVisible()}
                    onCheckedChange={(value) =>
                      column.toggleVisibility(!!value)
                    }
                    onSelect={(e) => e.preventDefault()}
                  >
                    {title(column.id)}
                  </DropdownMenuCheckboxItem>
                ))}
            </DropdownMenuContent>
          </DropdownMenu>
        ),
        cell: () => (
          <Button variant="ghost" className="h-8 w-8 p-0">
            <ChevronRight className="h-4 w-4" />
          </Button>
        ),
      }),
    ],
    [table],
  );

  return (
    <div className="h-full overflow-y-auto p-5">
      <div className="flex w-full flex-row justify-between pb-4">
        <div className="flex flex-row items-center space-x-3">
          <h2 className="text-2xl font-semibold leading-none">
            {title(tableId)}
          </h2>
          <TableSheet client={client} table={table} />
        </div>

        {table && <RowSheet table={table} />}
      </div>

      <DataTable
        customTable={table}
        data={data?.rows ?? []}
        columns={columns}
        sorting={
          lastTableId === tableId ? sorting : [{ id: 'createdAt', desc: true }]
        }
        setSorting={setSorting}
      />
    </div>
  );
}

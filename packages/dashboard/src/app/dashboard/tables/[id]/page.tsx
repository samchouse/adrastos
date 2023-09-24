'use client';

import { ColumnDef, createColumnHelper } from '@tanstack/react-table';
import { MoreHorizontal, PencilLine, Settings2, Trash2 } from 'lucide-react';
import { title } from 'radash';
import { useEffect, useMemo, useState } from 'react';

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '~/components';
import {
  useDeleteRowMutation,
  useTableDataQuery,
  useTablesQuery,
} from '~/hooks';

import { CreateRowSheet, DataTable } from './_components';

export type Row = { id: string } & Record<string, unknown>;

const columnHelper = createColumnHelper<Row>();

const Page: React.FC<{ params: { id: string } }> = ({ params }) => {
  const [cols, setCols] = useState<ColumnDef<Row>[]>([]);
  const { data: tables } = useTablesQuery();
  const { data } = useTableDataQuery<Row>(params.id);

  const { mutate } = useDeleteRowMutation(params.id);

  const table = useMemo(
    () => tables?.tables.find((t) => t.name === params.id),
    [tables, params.id],
  );

  useEffect(() => {
    setCols(
      [columnHelper.accessor('id', { header: 'Id' }) as ColumnDef<Row>]
        .concat(
          table?.fields.map((f) =>
            columnHelper.accessor(f.name, {
              header: title(f.name),
            }),
          ) ?? [],
        )
        .concat([
          columnHelper.display({
            id: 'actions',
            cell: ({ row }) => (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" className="h-8 w-8 p-0">
                    <span className="sr-only">Open menu</span>
                    <MoreHorizontal className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuLabel>Actions</DropdownMenuLabel>
                  <DropdownMenuItem>
                    <PencilLine className="mr-2 h-4 w-4" /> Edit
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => mutate(row.original.id)}>
                    <Trash2 className="mr-2 h-4 w-4" /> Delete
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            ),
          }),
        ]),
    );
  }, [table, mutate]);

  return (
    <div className="p-5">
      <div className="flex w-full flex-row justify-between pb-4">
        <div className="flex flex-row space-x-3">
          <h2 className="text-2xl font-semibold">{title(params.id)}</h2>
          <Button size="icon" variant="ghost">
            <Settings2 className="h-4 w-4" />
          </Button>
        </div>

        <CreateRowSheet
          table={table?.name ?? ''}
          fields={table?.fields ?? []}
        />
      </div>

      <DataTable data={data?.data ?? []} columns={cols} />
    </div>
  );
};

export default Page;

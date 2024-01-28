import { ColumnDef, createColumnHelper } from '@tanstack/react-table';
import { Settings2 } from 'lucide-react';
import { title } from 'radash';
import { useEffect, useMemo, useState } from 'react';

import { Button, Checkbox } from '~/components';
import { useTableDataQuery, useTablesQuery } from '~/hooks';

import { DataTable, RowSheet } from './_components';

export type Row = { id: string } & Record<string, unknown>;

const columnHelper = createColumnHelper<Row>();

const Page: React.FC<{ params: { id: string } }> = ({ params }) => {
  const [cols, setCols] = useState<ColumnDef<Row>[]>([]);
  const { data: tables } = useTablesQuery();
  const { data } = useTableDataQuery<Row>(params.id);

  const table = useMemo(
    () => tables?.find((t) => t.name === params.id),
    [tables, params.id],
  );

  useEffect(() => {
    setCols(
      [
        columnHelper.display({
          id: 'checkbox',
          meta: {
            style: {
              width: 'min',
            },
          },
          header: () => <Checkbox />,
          cell: () => <Checkbox />,
        }),
        columnHelper.accessor('id', { header: 'Id' }) as ColumnDef<Row>,
      ]
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
            meta: {
              style: {
                width: 'min',
                textAlign: 'right',
              },
            },
            cell: ({ row }) =>
              table && <RowSheet table={table} row={row.original} />,
          }),
        ]),
    );
  }, [table]);

  return (
    <div className="p-5">
      <div className="flex w-full flex-row justify-between pb-4">
        <div className="flex flex-row space-x-3">
          <h2 className="text-2xl font-semibold">{title(params.id)}</h2>
          <Button size="icon" variant="ghost">
            <Settings2 className="h-4 w-4" />
          </Button>
        </div>

        {table && <RowSheet table={table} />}
      </div>

      <DataTable data={data?.data ?? []} columns={cols} />
    </div>
  );
};

export default Page;

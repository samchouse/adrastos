import { CustomTable, Field } from '@adrastos/lib';
import { zodResolver } from '@hookform/resolvers/zod';
import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import {
  ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { ChevronRight, MoreHorizontal, Settings2, Trash2 } from 'lucide-react';
import { title } from 'radash';
import { useEffect, useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import {
  Button,
  Checkbox,
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input,
  Sheet,
  SheetClose,
  SheetContent,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '~/components';
import {
  tableDataQueryOptions,
  tablesQueryOptions,
  useCreateRowMutation,
  useDeleteRowMutation,
  useUpdateRowMutation,
} from '~/hooks';
import { cn } from '~/lib';

export const Route = createFileRoute('/dashboard/tables/$tableId')({
  component: TableIdComponent,
  loader: async ({
    context: { client, queryClient },
    params: { tableId },
  }) => ({
    tables: await queryClient.ensureQueryData(tablesQueryOptions(client)),
    data: queryClient.ensureQueryData(
      tableDataQueryOptions(client, tableId, false),
    ),
  }),
});

type Row = { id: string } & Record<string, unknown>;

const columnHelper = createColumnHelper<Row>();

const createFormSchema = (fields: Field[]) =>
  z.object(
    fields
      .map((f) => {
        let finalType: z.ZodTypeAny = z.any();
        switch (f.type) {
          case 'string': {
            let type = z.string();
            if (f.maxLength) type = type.max(f.maxLength);
            if (f.minLength) type = type.min(f.minLength);

            finalType = type;
            if (!f.isRequired) finalType = finalType.optional();
            break;
          }
          case 'number': {
            let type = z.coerce.number();
            if (f.max) type = type.max(f.max);
            if (f.min) type = type.min(f.min);

            finalType = type;
            break;
          }
          default:
        }

        return { [f.name]: finalType };
      })
      .reduce((a, b) => ({ ...a, ...b }), {}),
  );

export const RowSheet: React.FC<{
  row?: Row;
  table: CustomTable;
}> = ({ row, table }) => {
  const [isOpen, setIsOpen] = useState(false);
  const { client } = Route.useRouteContext();

  const { mutateAsync: createMutateAsync } = useCreateRowMutation(
    client,
    table.name,
  );
  const { mutateAsync: updateMutateAsync } = useUpdateRowMutation(
    client,
    table.name,
  );
  const { mutateAsync: deleteMutateAsync } = useDeleteRowMutation(
    client,
    table.name,
  );

  const formSchema = useMemo(
    () => createFormSchema(table.fields),
    [table.fields],
  );
  const form = useForm<z.infer<typeof formSchema>>({
    mode: 'onChange',
    resolver: zodResolver(formSchema),
    defaultValues:
      row &&
      table.fields.reduce(
        (a, b) => ({ ...a, [b.name]: row[b.name] ?? '' }),
        {},
      ),
  });

  useEffect(() => {
    if (row)
      form.reset(
        table.fields.reduce(
          (a, b) => ({ ...a, [b.name]: row[b.name] ?? '' }),
          {},
        ),
      );
  }, [row, table.fields, form]);

  return (
    <Sheet open={isOpen} onOpenChange={() => setIsOpen((o) => !o)}>
      <SheetTrigger asChild>
        {row ? (
          <Button variant="ghost" className="h-8 w-8 p-0">
            <ChevronRight className="h-4 w-4" />
          </Button>
        ) : (
          <Button size="sm" className="w-full">
            New Row
          </Button>
        )}
      </SheetTrigger>
      <SheetContent className="w-[500px] lg:max-w-[500px]">
        <Form {...form}>
          <form
            className="flex h-full flex-col justify-between"
            onSubmit={(e) =>
              void form.handleSubmit(async (values) => {
                if (row)
                  await updateMutateAsync({
                    match: { id: row.id },
                    data: values,
                  });
                else await createMutateAsync(values);

                form.reset();
                setIsOpen(false);
              })(e)
            }
          >
            <div>
              <SheetHeader>
                <SheetTitle>{row ? 'Edit' : 'Create'} A Row</SheetTitle>
              </SheetHeader>

              <div>
                {table.fields.map((f) => {
                  let field: React.ReactNode = null;
                  switch (f.type) {
                    case 'string':
                      field = (
                        <FormField
                          control={form.control}
                          name={f.name}
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>{title(f.name)}</FormLabel>
                              <FormControl>
                                <Input {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      );
                      break;
                    case 'number':
                      field = (
                        <FormField
                          control={form.control}
                          name={f.name}
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>{title(f.name)}</FormLabel>
                              <FormControl>
                                <Input type="number" {...field} />
                              </FormControl>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      );
                      break;
                    default:
                  }

                  return <div key={f.name}>{field}</div>;
                })}
              </div>
            </div>

            <SheetFooter className={cn(row && 'sm:justify-between')}>
              {row && (
                <Button
                  type="button"
                  variant="destructive"
                  onClick={() =>
                    void (async () => {
                      await deleteMutateAsync({ id: row.id });
                      setIsOpen(false);
                    })()
                  }
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete
                </Button>
              )}

              <div>
                <SheetClose asChild>
                  <Button variant="ghost">Cancel</Button>
                </SheetClose>
                <Button type="submit">Submit</Button>
              </div>
            </SheetFooter>
          </form>
        </Form>
      </SheetContent>
    </Sheet>
  );
};

const DataTable: React.FC<{
  data: Row[];
  columns: ColumnDef<Row>[];
}> = ({ columns, data }) => {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
  });

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead
                  key={header.id}
                  className={cn(
                    header.column.columnDef.meta?.style?.width === 'min' &&
                      'w-[1%]',
                  )}
                >
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && 'selected'}
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell
                    key={cell.id}
                    className={cn(
                      cell.column.columnDef.meta?.style?.textAlign &&
                        `text-${cell.column.columnDef.meta.style.textAlign}`,
                      cell.column.columnDef.meta?.style?.width === 'min' &&
                        'w-[1%]',
                    )}
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
};

function TableIdComponent() {
  const { client } = Route.useRouteContext();
  const { tableId } = Route.useParams();
  const [cols, setCols] = useState<ColumnDef<Row>[]>([]);

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

  useEffect(() => {
    setCols([
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
            aria-label="Select row"
          />
        ),
      }),
      columnHelper.accessor('id', {
        header: 'Id',
        enableHiding: false,
      }) as ColumnDef<Row>,
      ...(table?.fields.map((f) =>
        columnHelper.accessor(f.name, {
          header: title(f.name),
        }),
      ) ?? []),
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
                    className="capitalize"
                    checked={column.getIsVisible()}
                    onCheckedChange={(value) =>
                      column.toggleVisibility(!!value)
                    }
                    onSelect={(e) => e.preventDefault()}
                  >
                    {column.id}
                  </DropdownMenuCheckboxItem>
                ))}
            </DropdownMenuContent>
          </DropdownMenu>
        ),
        cell: ({ row }) =>
          table && <RowSheet table={table} row={row.original} />,
      }),
    ]);
  }, [table]);

  return (
    <div className="p-5">
      <div className="flex w-full flex-row justify-between pb-4">
        <div className="flex flex-row space-x-3">
          <h2 className="text-2xl font-semibold">{title(tableId)}</h2>
          <Button size="icon" variant="ghost">
            <Settings2 className="h-4 w-4" />
          </Button>
        </div>

        {table && <RowSheet table={table} />}
      </div>

      <DataTable data={data?.rows ?? []} columns={cols} />
    </div>
  );
}

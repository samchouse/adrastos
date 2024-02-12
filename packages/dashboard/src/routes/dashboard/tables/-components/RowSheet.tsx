import { CustomTable, Field } from '@adrastos/lib';
import { zodResolver } from '@hookform/resolvers/zod';
import { flexRender, Row as TableRowType } from '@tanstack/react-table';
import { Trash2 } from 'lucide-react';
import { title } from 'radash';
import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import {
  Button,
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
  TableCell,
  TableRow,
} from '~/components';
import {
  useCreateRowMutation,
  useDeleteRowMutation,
  useUpdateRowMutation,
} from '~/hooks';
import { cn } from '~/lib';

import { Row } from '.';
import { Route } from '../$tableId';

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

            finalType = z
              .string()
              .transform((v) => (v ? v : undefined))
              .pipe(f.isRequired ? type : type.optional());
            break;
          }
          case 'number': {
            let type = z.number();
            if (f.max) type = type.max(f.max);
            if (f.min) type = type.min(f.min);

            finalType = z
              .string()
              .transform((v) => (v ? parseFloat(v) : undefined))
              .pipe(f.isRequired ? type : type.optional());

            break;
          }
          default:
        }

        return { [f.name]: finalType };
      })
      .reduce((a, b) => ({ ...a, ...b }), {
        id: z
          .string()
          .optional()
          .transform((v) => (v ? v : undefined)),
      }),
  );

export const RowSheet: React.FC<{
  tableRow?: TableRowType<Row>;
  table: CustomTable;
  className?: string;
}> = ({ tableRow, table, className }) => {
  const [isOpen, setIsOpen] = useState(false);
  const { client } = Route.useRouteContext();
  const row = tableRow?.original;

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
  });

  return (
    <Sheet
      open={isOpen}
      onOpenChange={() => {
        setIsOpen((o) => !o);
        form.reset(
          table.fields.reduce(
            (a, b) => ({ ...a, [b.name]: row?.[b.name] ?? '' }),
            {
              id: row?.id ?? '',
            },
          ),
        );
      }}
    >
      <SheetTrigger asChild>
        {row ? (
          <TableRow
            key={tableRow.id}
            data-state={tableRow.getIsSelected() && 'selected'}
            className="cursor-pointer"
          >
            {tableRow.getVisibleCells().map((cell) => (
              <TableCell
                key={cell.id}
                className={cn(
                  'py-3',
                  cell.column.columnDef.meta?.style?.textAlign &&
                    `text-${cell.column.columnDef.meta.style.textAlign}`,
                  cell.column.columnDef.meta?.style?.width === 'min' &&
                    'w-min whitespace-nowrap',
                )}
              >
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </TableCell>
            ))}
          </TableRow>
        ) : (
          <Button size="sm" className={cn(className)}>
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

              <div className="mt-2 flex flex-col space-y-2">
                <FormField
                  disabled={!!row}
                  control={form.control}
                  name="id"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>{title('id')}</FormLabel>
                      <FormControl>
                        <Input
                          {...field}
                          placeholder="Leave empty to autogenerate"
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

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

              <div className="space-x-1">
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

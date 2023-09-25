import { zodResolver } from '@hookform/resolvers/zod';
import { ChevronRight, Trash2 } from 'lucide-react';
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
} from '~/components';
import { useCreateRowMutation, useDeleteRowMutation } from '~/hooks';
import { cn, Field, Table } from '~/lib';

import { Row } from '../page';

const createFormSchema = (fields: Field[]) =>
  z.object(
    fields
      .map((f) => {
        let finalType: z.ZodTypeAny = z.any();
        switch (f.type) {
          case 'string': {
            let type = z.string();
            if (f.isRequired) type = type.nonempty();
            if (f.maxLength) type = type.max(f.maxLength);
            if (f.minLength) type = type.min(f.minLength);

            finalType = type;
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
  table: Table;
}> = ({ row, table }) => {
  const [isOpen, setIsOpen] = useState(false);

  const { mutateAsync: createMutateAsync } = useCreateRowMutation(table.name);
  const { mutateAsync: deleteMutateAsync } = useDeleteRowMutation(table.name);

  const formSchema = useMemo(
    () => createFormSchema(table.fields),
    [table.fields],
  );
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    mode: 'onChange',
    defaultValues:
      row &&
      table.fields.reduce(
        (a, b) => ({ ...a, [b.name]: row[b.name] ?? '' }),
        {},
      ),
  });

  return (
    <Sheet open={isOpen} onOpenChange={() => setIsOpen((o) => !o)}>
      <SheetTrigger asChild>
        {row ? (
          <Button variant="ghost" className="h-8 w-8 p-0">
            <ChevronRight className="h-4 w-4" />
          </Button>
        ) : (
          <Button className="mb-3 w-full">New Row</Button>
        )}
      </SheetTrigger>
      <SheetContent className="w-[500px] lg:max-w-[500px]">
        <Form {...form}>
          <form
            className="flex h-full flex-col justify-between"
            onSubmit={form.handleSubmit(async (values) => {
              await createMutateAsync(values);
              form.reset();
              setIsOpen(false);
            })}
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
                  onClick={async () => {
                    await deleteMutateAsync(row.id);
                    setIsOpen(false);
                  }}
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

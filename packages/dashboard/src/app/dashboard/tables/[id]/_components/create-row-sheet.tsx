import { zodResolver } from '@hookform/resolvers/zod';
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
import { useCreateRowMutation } from '~/hooks';
import { Field } from '~/lib';

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

export const CreateRowSheet: React.FC<{ table: string; fields: Field[] }> = ({
  table,
  fields,
}) => {
  const [isOpen, setIsOpen] = useState(false);

  const { mutateAsync } = useCreateRowMutation(table);

  const formSchema = useMemo(() => createFormSchema(fields), [fields]);
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    mode: 'onChange',
  });

  return (
    <Sheet open={isOpen} onOpenChange={() => setIsOpen((o) => !o)}>
      <SheetTrigger asChild>
        <Button className="mb-3 w-full">New Row</Button>
      </SheetTrigger>
      <SheetContent className="w-[500px] lg:max-w-[500px]">
        <Form {...form}>
          <form
            className="flex h-full flex-col justify-between"
            onSubmit={form.handleSubmit(async (values) => {
              await mutateAsync(values);
              form.reset();
              setIsOpen(false);
            })}
          >
            <div>
              <SheetHeader>
                <SheetTitle>Create A Row</SheetTitle>
              </SheetHeader>

              <div>
                {fields.map((f) => {
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

            <SheetFooter>
              <SheetClose asChild>
                <Button variant="ghost">Cancel</Button>
              </SheetClose>
              <Button type="submit">Submit</Button>
            </SheetFooter>
          </form>
        </Form>
      </SheetContent>
    </Sheet>
  );
};

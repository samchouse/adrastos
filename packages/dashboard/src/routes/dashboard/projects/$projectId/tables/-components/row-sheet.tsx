import type { Client, CustomTable, Field } from '@adrastos/lib';
import { zodResolver } from '@hookform/resolvers/zod';
import { useQueries } from '@tanstack/react-query';
import { type Row as TableRowType, flexRender } from '@tanstack/react-table';
import { CommandList } from 'cmdk';
import { Check, ChevronsUpDown, Info, Trash2 } from 'lucide-react';
import { title } from 'radash';
import { useEffect, useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import {
  Button,
  Checkbox,
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  DateTimePicker,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input,
  Label,
  MultiSelect,
  type Option,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Sheet,
  SheetClose,
  SheetContent,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
  Switch,
  TableCell,
  TableRow,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '~/components';
import {
  tableDataQueryOptions,
  useCreateRowMutation,
  useDeleteRowMutation,
  useUpdateRowMutation,
} from '~/hooks';
import { cn } from '~/lib';

import type { Row } from '.';

function createFormSchema(fields: Field[]) {
  return z.object(
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
              .transform((v) => (v ? Number.parseFloat(v) : undefined))
              .pipe(f.isRequired ? type : type.optional());
            break;
          }
          case 'boolean': {
            finalType = z.coerce.boolean();
            break;
          }
          case 'date': {
            const type = z.date();
            finalType = type.pipe(f.isRequired ? type : type.optional());
            break;
          }
          case 'email': {
            const type = z.string().email();
            finalType = type.pipe(f.isRequired ? type : type.optional());
            break;
          }
          case 'url': {
            const type = z.string().url();
            finalType = type.pipe(f.isRequired ? type : type.optional());
            break;
          }
          case 'select': {
            let type = z.array(z.string());
            if (f.maxSelected) type = type.max(f.maxSelected);
            if (f.minSelected) type = type.min(f.minSelected);

            finalType = (
              f.maxSelected === 1 && f.minSelected === 1
                ? z.coerce.string().transform((v) => [v])
                : type
            ).pipe(f.isRequired ? type : type.optional());
            break;
          }
          case 'relation': {
            let type: z.ZodTypeAny;
            if (f.target === 'many') {
              type = z.array(z.string());
            } else {
              type = z.string().transform((v) => (v ? v : undefined));
            }

            finalType = type.pipe(f.isRequired ? type : type.optional());
            break;
          }
          default:
        }

        return { [f.name]: finalType };
      })
      .reduce(
        (a, b) => ({
          ...a,
          ...b,
        }),
        {
          id: z
            .string()
            .optional()
            .transform((v) => (v ? v : undefined)),
        },
      ),
  );
}

const RelationPicker: React.FC<{
  multiple: boolean;
  table: string;
  client: Client;
  values: string[];
  onSave: (values: string | string[]) => void;
}> = ({ multiple, table, client, onSave, values }) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selected, setSelected] = useState<string[]>(values);

  const [{ data: tableData }] = useQueries({
    queries: [tableDataQueryOptions<Row, false>(client, table, false)],
  });

  const [hasReset, setHasReset] = useState(false);
  useEffect(() => {
    if (values.length !== 0) setHasReset(false);
    if (values.length === 0 && !hasReset) {
      setHasReset(true);
      onSave(multiple ? [] : '');
    }
  }, [hasReset, multiple, onSave, values]);

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <div className="overflow-hidden rounded-md border text-sm">
        {values.length === 0 ? (
          <div className="flex justify-center p-2 text-muted-foreground">
            No rows selected
          </div>
        ) : (
          <div>
            {values.map((value, index) => (
              <div
                key={index}
                className={cn(
                  'flex flex-row items-center justify-between py-1 pr-1 pl-3',
                  index !== values.length - 1 && 'border-b',
                )}
              >
                <div className="flex flex-row space-x-2">
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger>
                        <Info className="size-4" />
                      </TooltipTrigger>
                      <TooltipContent className="whitespace-pre-wrap">
                        {JSON.stringify(
                          tableData?.rows.find((r) => r.id === value),
                          null,
                          4,
                        )}
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                  <p>{value}</p>
                </div>

                <Button
                  size="icon"
                  type="button"
                  variant="ghost"
                  className="size-8"
                  onClick={() => {
                    setSelected((values) => values.filter((v) => v !== value));
                    onSave(multiple ? selected.filter((v) => v !== value) : '');
                  }}
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
            ))}
          </div>
        )}

        <DialogTrigger asChild>
          <Button size="sm" variant="secondary" className="w-full rounded-none">
            Open selector
          </Button>
        </DialogTrigger>
      </div>

      <DialogContent className="w-full sm:max-w-[425px] md:max-w-[700px]">
        <DialogHeader>
          <DialogTitle>Select rows</DialogTitle>
        </DialogHeader>

        <div>
          {tableData?.rows.map((row, index, arr) => (
            <div
              key={index}
              className={cn(
                'flex w-full cursor-pointer flex-row items-center justify-between border-x border-t p-4 hover:bg-secondary',
                index === 0 && 'rounded-t-md',
                index === arr.length - 1 && 'rounded-b-md border-b',
              )}
              onClick={() => {
                multiple
                  ? setSelected((values) =>
                      values.includes(row.id)
                        ? values.filter((val) => val !== row.id)
                        : [...values, row.id],
                    )
                  : setSelected([row.id]);
              }}
            >
              <div className="flex flex-row items-center space-x-3">
                <Checkbox
                  id={index.toString()}
                  checked={selected.includes(row.id)}
                />
                <Label
                  htmlFor={index.toString()}
                  className="cursor-pointer"
                  onClick={() => {
                    multiple
                      ? setSelected((values) =>
                          values.includes(row.id)
                            ? values.filter((val) => val !== row.id)
                            : [...values, row.id],
                        )
                      : setSelected([row.id]);
                  }}
                >
                  {row.id}
                </Label>
              </div>

              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger>
                    <Info className="size-4" />
                  </TooltipTrigger>
                  <TooltipContent className="whitespace-pre-wrap">
                    {JSON.stringify(row, null, 4)}
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
          ))}
        </div>

        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => {
              setIsOpen(false);
            }}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            onClick={() => {
              setIsOpen(false);
              onSave(multiple ? selected : selected[0]);
            }}
          >
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const SingleSelect: React.FC<{
  options: Option[];
  name?: string;
  disabled?: boolean;
  value?: string;
  onBlur?: () => void;
  onSelect?: (value: string) => void;
}> = ({ options, disabled, onSelect, onBlur, name, value: passedValue }) => {
  const [open, setOpen] = useState(false);
  const [value, setValue] = useState(passedValue ?? '');

  return (
    <Popover
      open={open}
      onOpenChange={(o) => {
        setOpen(o);
        if (!o) onBlur?.();
      }}
    >
      <PopoverTrigger asChild>
        <Button
          name={name}
          role="combobox"
          variant="outline"
          disabled={disabled}
          aria-expanded={open}
          className="w-full justify-between"
        >
          {value
            ? options.find((option) => option.value === value)?.label
            : 'Select option...'}
          <ChevronsUpDown className="ml-2 size-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[451px] p-0">
        <Command>
          <CommandInput placeholder="Search options..." />
          <CommandEmpty>No option found.</CommandEmpty>

          <CommandList>
            <CommandGroup>
              {options.map((option) => (
                <CommandItem
                  key={option.value}
                  value={option.value}
                  onSelect={(currentValue) => {
                    const newValue = currentValue === value ? '' : currentValue;
                    setValue(newValue);
                    onSelect?.(newValue);
                    setOpen(false);
                  }}
                >
                  <Check
                    className={cn(
                      'mr-2 size-4',
                      value === option.value ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                  {option.label}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
};

export const RowSheet: React.FC<{
  tableRow?: TableRowType<Row>;
  table: CustomTable;
  className?: string;
  client: Client;
}> = ({ tableRow, table, className, client }) => {
  const [isOpen, setIsOpen] = useState(false);
  const row = tableRow?.original;

  const { mutateAsync: createMutateAsync } = useCreateRowMutation(table.name);
  const { mutateAsync: updateMutateAsync } = useUpdateRowMutation(table.name);
  const { mutateAsync: deleteMutateAsync } = useDeleteRowMutation(table.name);

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
            (a, b) => ({
              ...a,
              [b.name]: row?.[b.name] ?? '',
            }),
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
            className="cursor-pointer"
            data-state={tableRow.getIsSelected() && 'selected'}
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
                  name="id"
                  disabled={!!row}
                  control={form.control}
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
                          name={f.name}
                          control={form.control}
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
                          name={f.name}
                          control={form.control}
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
                    case 'boolean':
                      field = (
                        <FormField
                          name={f.name}
                          control={form.control}
                          render={({ field }) => (
                            <FormItem className="mt-1 flex items-center space-x-2">
                              <FormControl>
                                <Switch
                                  size="sm"
                                  {...{
                                    ...field,
                                    value: undefined,
                                    onChange: undefined,
                                  }}
                                  checked={field.value as boolean}
                                  onCheckedChange={field.onChange}
                                />
                              </FormControl>
                              <FormLabel>{title(f.name)}</FormLabel>
                            </FormItem>
                          )}
                        />
                      );
                      break;
                    case 'date':
                      field = (
                        <FormField
                          name={f.name}
                          control={form.control}
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel htmlFor={undefined}>
                                {title(f.name)}
                              </FormLabel>
                              <FormControl>
                                <DateTimePicker
                                  {...{
                                    ...field,
                                    value: undefined,
                                    onChange: undefined,
                                  }}
                                  granularity="second"
                                  onJsDateChange={field.onChange}
                                  jsDate={
                                    typeof field.value === 'string' &&
                                    field.value !== ''
                                      ? new Date(field.value)
                                      : (field.value as Date)
                                  }
                                />
                              </FormControl>
                            </FormItem>
                          )}
                        />
                      );
                      break;
                    case 'email':
                      field = (
                        <FormField
                          name={f.name}
                          control={form.control}
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>{title(f.name)}</FormLabel>
                              <FormControl>
                                <Input {...field} />
                              </FormControl>
                            </FormItem>
                          )}
                        />
                      );
                      break;
                    case 'url':
                      field = (
                        <FormField
                          name={f.name}
                          control={form.control}
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>{title(f.name)}</FormLabel>
                              <FormControl>
                                <Input {...field} />
                              </FormControl>
                            </FormItem>
                          )}
                        />
                      );
                      break;
                    case 'select':
                      field = (
                        <FormField
                          name={f.name}
                          control={form.control}
                          render={({ field }) => {
                            const options = f.options.reduce<Option[]>(
                              (acc, curr) => [
                                ...acc,
                                {
                                  label: title(curr),
                                  value: curr,
                                },
                              ],
                              [],
                            );

                            return (
                              <FormItem>
                                <FormLabel>{title(f.name)}</FormLabel>
                                <FormControl>
                                  {f.minSelected === 1 &&
                                  f.maxSelected === 1 ? (
                                    <SingleSelect
                                      {...{
                                        ...field,
                                        ref: undefined,
                                        value: undefined,
                                        onChange: undefined,
                                      }}
                                      options={options}
                                      onSelect={(value) => {
                                        form.setValue(field.name, value);
                                      }}
                                      value={
                                        Array.isArray(field.value)
                                          ? (field.value[0] as string)
                                          : (field.value as string)
                                      }
                                    />
                                  ) : (
                                    <MultiSelect
                                      {...{
                                        ...field,
                                        value: undefined,
                                        onChange: undefined,
                                      }}
                                      options={options}
                                      placeholder="Select options..."
                                      onSelectedChange={(values) => {
                                        form.setValue(
                                          field.name,
                                          values.map((v) => v.value),
                                        );
                                      }}
                                      selected={
                                        field.value === ''
                                          ? []
                                          : ((field.value as string[]).map(
                                              (v) => ({
                                                label: title(v),
                                                value: v,
                                              }),
                                            ) satisfies Option[])
                                      }
                                    />
                                  )}
                                </FormControl>
                              </FormItem>
                            );
                          }}
                        />
                      );
                      break;
                    case 'relation':
                      field = (
                        <FormField
                          name={f.name}
                          control={form.control}
                          render={({ field }) => (
                            <FormItem>
                              <FormLabel>{title(f.name)}</FormLabel>
                              <FormControl>
                                <RelationPicker
                                  {...{
                                    ...field,
                                    ref: undefined,
                                  }}
                                  table={f.table}
                                  client={client}
                                  multiple={f.target === 'many'}
                                  onSave={(values) => {
                                    form.setValue(f.name, values);
                                  }}
                                  values={
                                    typeof field.value === 'string'
                                      ? field.value === ''
                                        ? []
                                        : [field.value]
                                      : [
                                          field.value as (
                                            | string
                                            | { id: string }
                                          )[],
                                        ]
                                          .flat()
                                          .map((v) =>
                                            typeof v === 'string' ? v : v.id,
                                          )
                                  }
                                />
                              </FormControl>
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
                  <Trash2 className="mr-2 size-4" />
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

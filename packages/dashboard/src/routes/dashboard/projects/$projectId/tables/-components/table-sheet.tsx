import { Client, CustomTable, FieldCrud } from '@adrastos/lib';
import { zodResolver } from '@hookform/resolvers/zod';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate, useParams } from '@tanstack/react-router';
import {
  Calendar,
  Database,
  Hash,
  Link2,
  List,
  Lock,
  LockOpen,
  Plus,
  Settings2,
  ToggleRight,
  Trash2,
  Type,
} from 'lucide-react';
import { omit, title } from 'radash';
import { useCallback, useState } from 'react';
import isEqual from 'react-fast-compare';
import {
  FieldArray,
  useFieldArray,
  UseFieldArrayUpdate,
  useForm,
  UseFormReturn,
} from 'react-hook-form';
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
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Sheet,
  SheetClose,
  SheetContent,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '~/components';
import {
  tablesQueryOptions,
  useCreateTableMutation,
  useDeleteTableMutation,
  useUpdateTableMutation,
} from '~/hooks';
import { cn } from '~/lib';

const formSchema = z.object({
  name: z.string(),
  fields: z.array(
    z.union([
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.literal('string'),
        minLength: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        maxLength: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        pattern: z.string().nullable(),
        isRequired: z.boolean(),
        isUnique: z.boolean(),
      }),
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.literal('number'),
        min: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        max: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        isRequired: z.boolean(),
        isUnique: z.boolean(),
      }),
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.literal('boolean'),
      }),
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.literal('date'),
        isRequired: z.boolean(),
        isUnique: z.boolean(),
      }),
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.union([z.literal('email'), z.literal('url')]),
        except: z.array(z.string()),
        only: z.array(z.string()),
        isRequired: z.boolean(),
        isUnique: z.boolean(),
      }),
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.literal('select'),
        options: z.array(z.string()),
        minSelected: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        maxSelected: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        isRequired: z.boolean(),
        isUnique: z.boolean(),
      }),
      z.object({
        originalName: z.string().optional(),
        name: z.string(),
        type: z.literal('relation'),
        target: z.union([z.literal('single'), z.literal('many')]),
        table: z.string(),
        cascadeDelete: z.boolean(),
        minSelected: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        maxSelected: z
          .string()
          .transform((v) => (v ? parseInt(v, 10) : null))
          .nullable(),
        isRequired: z.boolean(),
        isUnique: z.boolean(),
      }),
    ]),
  ),
  permissions: z.object({
    view: z.string().nullable(),
    create: z.string().nullable(),
    update: z.string().nullable(),
    delete: z.string().nullable(),
  }),
});

const FieldCard: React.FC<
  React.PropsWithChildren<{
    index: number;
    form: UseFormReturn<z.infer<typeof formSchema>>;
    update: UseFieldArrayUpdate<z.infer<typeof formSchema>, 'fields'>;
    field: FieldArray<z.infer<typeof formSchema>, 'fields'>;
    toggles?: JSX.Element;
  }>
> = ({ children, form, index, update, field: f, toggles }) => (
  <div className="mt-2 grid grid-cols-2 gap-2">
    <FormField
      control={form.control}
      name={`fields.${index}.name`}
      render={({ field }) => (
        <FormItem className="col-span-2">
          <FormLabel>Name</FormLabel>
          <FormControl>
            <Input {...field} placeholder="Name" data-form-type="other" />
          </FormControl>
          <FormMessage />
        </FormItem>
      )}
    />

    {children}

    {f.type !== 'boolean' && (
      <div className="col-span-2 mt-2 flex flex-row space-x-5">
        <FormField
          control={form.control}
          name={`fields.${index}.isRequired`}
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center space-x-2">
                <FormControl>
                  <Switch
                    {...{
                      ...field,
                      value: undefined,
                      onChange: undefined,
                    }}
                    size="sm"
                    checked={field.value}
                    onCheckedChange={(checked) =>
                      update(index, {
                        ...f,
                        isRequired: checked,
                      })
                    }
                  />
                </FormControl>
                <FormLabel>Required</FormLabel>
              </div>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name={`fields.${index}.isUnique`}
          render={({ field }) => (
            <FormItem>
              <div className="flex items-center space-x-2">
                <FormControl>
                  <Switch
                    {...{
                      ...field,
                      value: undefined,
                      onChange: undefined,
                    }}
                    size="sm"
                    checked={field.value}
                    onCheckedChange={(checked) =>
                      update(index, {
                        ...f,
                        isUnique: checked,
                      })
                    }
                  />
                </FormControl>
                <FormLabel>Unique</FormLabel>
              </div>
              <FormMessage />
            </FormItem>
          )}
        />

        {toggles}
      </div>
    )}
  </div>
);

type Types<T = z.infer<typeof formSchema>['fields']> = T extends (infer U)[]
  ? U extends { type: string }
    ? U['type']
    : never
  : never;

type Properties<
  T extends Types,
  U = z.infer<typeof formSchema>['fields'],
> = U extends (infer V)[]
  ? V extends unknown
    ? Extract<V, { type: T }> extends never
      ? never
      : Exclude<
          {
            [K in keyof V]: number extends V[K] ? K : never;
          }[keyof V],
          undefined
        >
    : never
  : never;

const NumberInput = <T extends Types>({
  form,
  index,
  property,
}: {
  index: number;
  property: Properties<T>;
  form: UseFormReturn<z.infer<typeof formSchema>>;
}) => (
  <FormField
    control={form.control}
    name={`fields.${index}.${property}`}
    render={({ field }) => (
      <FormItem>
        <FormLabel>{title(property)}</FormLabel>
        <FormControl>
          <Input
            {...field}
            type="number"
            value={field.value ?? ''}
            placeholder={title(property)}
            data-form-type="other"
          />
        </FormControl>
        <FormMessage />
      </FormItem>
    )}
  />
);

export const TableSheet: React.FC<{
  client: Client;
  table?: CustomTable;
  className?: string;
  tables: CustomTable[];
}> = ({ client, table, className, tables }) => {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const params = useParams({
    from: '/dashboard/projects/$projectId',
  });

  const [isOpen, setIsOpen] = useState(false);
  const [isOpenPopover, setIsOpenPopover] = useState(false);

  const { mutateAsync: createMutateAsync } = useCreateTableMutation();
  const { mutateAsync: updateMutateAsync } = useUpdateTableMutation();
  const { mutateAsync: deleteMutateAsync } = useDeleteTableMutation();

  const form = useForm<z.infer<typeof formSchema>>({
    mode: 'onChange',
    resolver: zodResolver(formSchema),
  });
  const { fields, append, update, remove } = useFieldArray({
    name: 'fields',
    control: form.control,
  });

  const lowestId = useCallback(() => {
    const usedSuffixes = fields
      .filter(
        (field) =>
          new RegExp(/field\d*/).exec(field.name)?.[0].length ===
          field.name.length,
      )
      .map((field) => field.name.replace('field', ''))
      .concat(
        table?.fields
          .filter(
            (field) =>
              new RegExp(/field\d*/).exec(field.name)?.[0].length ===
              field.name.length,
          )
          .map((field) => field.name.replace('field', '')) ?? [],
      )
      .reduce(
        (acc, curr) => (acc.includes(curr) ? acc : [...acc, curr]),
        [] as string[],
      )
      .map((suffix) => (suffix === '' ? 0 : parseInt(suffix, 10)))
      .sort();

    const varianceSuffix = usedSuffixes.findIndex((s, i) => s !== i);
    const lastSuffix = usedSuffixes.pop();
    return `field${varianceSuffix !== -1 ? (varianceSuffix === 0 ? '' : varianceSuffix) : lastSuffix !== undefined ? lastSuffix + 1 : ''}`;
  }, [fields, table]);

  return (
    <Sheet
      open={isOpen}
      onOpenChange={() => {
        setIsOpen((o) => !o);
        setIsOpenPopover(false);

        form.reset({
          name: table?.name ?? '',
          fields:
            table?.fields.map((f) => ({ ...f, originalName: f.name })) ?? [],
          permissions: table?.permissions ?? {
            view: null,
            create: null,
            update: null,
            delete: null,
          },
        });
      }}
    >
      <SheetTrigger asChild>
        {table ? (
          <Button size="icon" variant="ghost">
            <Settings2 className="h-4 w-4" />
          </Button>
        ) : (
          <Button className={cn('mb-3 w-full', className)}>
            <Plus className="mr-2 h-4 w-4" /> Create New
          </Button>
        )}
      </SheetTrigger>
      <SheetContent className="w-[500px] lg:max-w-[500px]">
        <Form {...form}>
          <form
            className="flex h-full flex-col justify-between"
            onSubmit={(e) =>
              void form.handleSubmit(async (values) => {
                let tableName = '';
                if (table) {
                  const updatedTable = await updateMutateAsync({
                    name: table.name,
                    data: {
                      ...values,
                      fields: (
                        values.fields
                          .filter((field) => {
                            const omittedField = omit(field, ['originalName']);
                            const previousField = table.fields.find(
                              (f) => f.name === field.originalName,
                            );

                            return !isEqual(omittedField, previousField);
                          })
                          .map((field) => {
                            const previousField = table.fields.find(
                              (f) => f.name === field.originalName,
                            );
                            return {
                              name: previousField
                                ? previousField.name
                                : field.name,
                              action: previousField ? 'update' : 'create',
                              field: field,
                            };
                          }) as FieldCrud[]
                      ).concat(
                        table.fields
                          .filter(
                            (field) =>
                              !values.fields.some(
                                (f) => f.originalName === field.name,
                              ),
                          )
                          .map((field) => ({
                            name: field.name,
                            action: 'delete',
                          })) as FieldCrud[],
                      ),
                    },
                  });
                  tableName = updatedTable.name;
                } else {
                  const table = await createMutateAsync(values);
                  tableName = table.name;
                }

                setIsOpen(false);
                await navigate({
                  to: '/dashboard/projects/$projectId/tables/$tableId',
                  params: {
                    projectId: params.projectId,
                    tableId: tableName,
                  },
                });
              })(e)
            }
          >
            <div className="h-full">
              <SheetHeader className="mb-5">
                <SheetTitle>Create A New Table</SheetTitle>
              </SheetHeader>

              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem className="mb-3">
                    <FormLabel>Name</FormLabel>
                    <FormControl>
                      <Input {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />

              <Tabs defaultValue="fields">
                <TabsList className="grid w-full grid-cols-2">
                  <TabsTrigger value="fields">Fields</TabsTrigger>
                  <TabsTrigger value="permissions">Permissions</TabsTrigger>
                </TabsList>
                <TabsContent value="fields">
                  {fields.length > 0 && (
                    <div className="mb-3 max-h-[calc(100vh-330px)] space-y-3 overflow-auto">
                      {fields.map((_, index) => {
                        const f = form.watch(`fields.${index}`);

                        let field: React.ReactNode = null;
                        switch (f.type) {
                          case 'string':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              >
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.pattern`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Pattern</FormLabel>
                                      <FormControl>
                                        <Input
                                          {...field}
                                          value={field.value ?? ''}
                                          placeholder="Pattern"
                                          data-form-type="other"
                                        />
                                      </FormControl>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />

                                <NumberInput<'string'>
                                  form={form}
                                  index={index}
                                  property="minLength"
                                />
                                <NumberInput<'string'>
                                  form={form}
                                  index={index}
                                  property="maxLength"
                                />
                              </FieldCard>
                            );
                            break;
                          case 'number':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              >
                                <NumberInput<'number'>
                                  form={form}
                                  index={index}
                                  property="min"
                                />
                                <NumberInput<'number'>
                                  form={form}
                                  index={index}
                                  property="max"
                                />
                              </FieldCard>
                            );
                            break;
                          case 'date':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              />
                            );
                            break;
                          case 'email':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              >
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.except`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Except</FormLabel>
                                      <FormControl>
                                        <Input
                                          {...field}
                                          value={field.value?.join(', ') ?? ''}
                                          placeholder="Except"
                                          data-form-type="other"
                                          onChange={(e) =>
                                            form.setValue(
                                              `fields.${index}.except`,
                                              e.target.value
                                                .replaceAll(' ', '')
                                                .split(','),
                                            )
                                          }
                                          disabled={fields.some(
                                            (field) =>
                                              field.originalName ===
                                                f.originalName &&
                                              field.type === 'email' &&
                                              field.only.length > 0,
                                          )}
                                        />
                                      </FormControl>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.only`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Only</FormLabel>
                                      <FormControl>
                                        <Input
                                          {...field}
                                          value={field.value?.join(', ') ?? ''}
                                          placeholder="Only"
                                          data-form-type="other"
                                          onChange={(e) =>
                                            form.setValue(
                                              `fields.${index}.only`,
                                              e.target.value
                                                .replaceAll(' ', '')
                                                .split(','),
                                            )
                                          }
                                          disabled={fields.some(
                                            (field) =>
                                              field.originalName ===
                                                f.originalName &&
                                              field.type === 'email' &&
                                              field.except.length > 0,
                                          )}
                                        />
                                      </FormControl>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />
                              </FieldCard>
                            );
                            break;
                          case 'url':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              >
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.except`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Except</FormLabel>
                                      <FormControl>
                                        <Input
                                          {...field}
                                          value={field.value?.join(', ') ?? ''}
                                          placeholder="Except"
                                          data-form-type="other"
                                          onChange={(e) =>
                                            form.setValue(
                                              `fields.${index}.except`,
                                              e.target.value
                                                .replaceAll(' ', '')
                                                .split(','),
                                            )
                                          }
                                          disabled={fields.some(
                                            (field) =>
                                              field.originalName ===
                                                f.originalName &&
                                              field.type === 'url' &&
                                              field.only.length > 0,
                                          )}
                                        />
                                      </FormControl>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.only`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Only</FormLabel>
                                      <FormControl>
                                        <Input
                                          {...field}
                                          value={field.value?.join(', ') ?? ''}
                                          placeholder="Only"
                                          data-form-type="other"
                                          onChange={(e) =>
                                            form.setValue(
                                              `fields.${index}.only`,
                                              e.target.value
                                                .replaceAll(' ', '')
                                                .split(','),
                                            )
                                          }
                                          disabled={fields.some(
                                            (field) =>
                                              field.originalName ===
                                                f.originalName &&
                                              field.type === 'url' &&
                                              field.except.length > 0,
                                          )}
                                        />
                                      </FormControl>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />
                              </FieldCard>
                            );
                            break;
                          case 'select':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              >
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.options`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Options</FormLabel>
                                      <FormControl>
                                        <Input
                                          {...field}
                                          value={field.value.join(', ') ?? ''}
                                          placeholder="Options"
                                          data-form-type="other"
                                          onChange={(e) =>
                                            form.setValue(
                                              `fields.${index}.options`,
                                              e.target.value
                                                .replaceAll(' ', '')
                                                .split(','),
                                            )
                                          }
                                        />
                                      </FormControl>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />

                                <NumberInput<'select'>
                                  form={form}
                                  index={index}
                                  property="minSelected"
                                />
                                <NumberInput<'select'>
                                  form={form}
                                  index={index}
                                  property="maxSelected"
                                />
                              </FieldCard>
                            );
                            break;
                          case 'relation':
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                                toggles={
                                  <FormField
                                    control={form.control}
                                    name={`fields.${index}.cascadeDelete`}
                                    render={({ field }) => (
                                      <FormItem>
                                        <div className="flex items-center space-x-2">
                                          <FormControl>
                                            <Switch
                                              {...{
                                                ...field,
                                                value: undefined,
                                                onChange: undefined,
                                              }}
                                              size="sm"
                                              checked={f.cascadeDelete}
                                              onCheckedChange={(checked) =>
                                                update(index, {
                                                  ...f,
                                                  cascadeDelete: checked,
                                                })
                                              }
                                            />
                                          </FormControl>
                                          <FormLabel>Cascade Delete</FormLabel>
                                        </div>
                                        <FormMessage />
                                      </FormItem>
                                    )}
                                  />
                                }
                              >
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.table`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Table</FormLabel>
                                      <Select
                                        disabled={!!table}
                                        defaultValue={field.value}
                                        onValueChange={field.onChange}
                                        onOpenChange={(c) =>
                                          !c && field.onBlur()
                                        }
                                      >
                                        <FormControl>
                                          <SelectTrigger>
                                            <SelectValue placeholder="Select table" />
                                          </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                          {tables.map(
                                            (t) =>
                                              t.id !== table?.id && (
                                                <SelectItem
                                                  key={t.id}
                                                  value={t.name}
                                                >
                                                  {title(t.name)}
                                                </SelectItem>
                                              ),
                                          )}
                                        </SelectContent>
                                      </Select>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />
                                <FormField
                                  control={form.control}
                                  name={`fields.${index}.target`}
                                  render={({ field }) => (
                                    <FormItem className="col-span-2">
                                      <FormLabel>Target</FormLabel>
                                      <Select
                                        defaultValue={field.value}
                                        onValueChange={field.onChange}
                                        onOpenChange={(c) =>
                                          !c && field.onBlur()
                                        }
                                      >
                                        <FormControl>
                                          <SelectTrigger>
                                            <SelectValue />
                                          </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                          <SelectItem value="single">
                                            Single
                                          </SelectItem>
                                          <SelectItem value="many">
                                            Many
                                          </SelectItem>
                                        </SelectContent>
                                      </Select>
                                      <FormMessage />
                                    </FormItem>
                                  )}
                                />

                                {f.target === 'many' ? (
                                  <>
                                    <NumberInput<'relation'>
                                      form={form}
                                      index={index}
                                      property="minSelected"
                                    />
                                    <NumberInput<'select'>
                                      form={form}
                                      index={index}
                                      property="maxSelected"
                                    />
                                  </>
                                ) : null}
                              </FieldCard>
                            );
                            break;
                          default:
                            field = (
                              <FieldCard
                                field={f}
                                form={form}
                                index={index}
                                update={update}
                              />
                            );
                        }

                        return (
                          <div
                            key={index}
                            className="rounded-md border p-3 pt-2"
                          >
                            <div className="flex flex-row items-center justify-between">
                              <h3 className="text-base font-medium">
                                {title(f.type)} Field
                              </h3>
                              <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                onClick={() => remove(index)}
                              >
                                <Trash2 className="h-4 w-4" />
                              </Button>
                            </div>

                            {field}
                          </div>
                        );
                      })}
                    </div>
                  )}

                  <Popover
                    open={isOpenPopover}
                    onOpenChange={() => setIsOpenPopover((o) => !o)}
                  >
                    <PopoverTrigger asChild>
                      <Button className="w-full" variant="secondary">
                        Add field
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent
                      className="w-[451px] bg-background"
                      sideOffset={8}
                    >
                      <div className="grid grid-cols-6 gap-3 rounded-md">
                        <div className="col-span-2 h-14">
                          <Button
                            variant="secondary"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'string',
                                isRequired: false,
                                isUnique: false,
                                maxLength: null,
                                minLength: null,
                                pattern: null,
                              });
                            }}
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                          >
                            <Type className="h-6 w-6" />
                            String
                          </Button>
                        </div>
                        <div className="col-span-2 h-14">
                          <Button
                            variant="secondary"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'number',
                                isRequired: false,
                                isUnique: false,
                                max: null,
                                min: null,
                              });
                            }}
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                          >
                            <Hash className="h-6 w-6" />
                            Number
                          </Button>
                        </div>
                        <div className="col-span-2 h-14">
                          <Button
                            variant="secondary"
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'boolean',
                              });
                            }}
                          >
                            <ToggleRight className="h-6 w-6" />
                            Boolean
                          </Button>
                        </div>
                        <div className="col-span-2 h-14">
                          <Button
                            variant="secondary"
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'date',
                                isRequired: false,
                                isUnique: false,
                              });
                            }}
                          >
                            <Calendar className="h-6 w-6" />
                            Date
                          </Button>
                        </div>
                        <div className="col-span-2 h-14">
                          <Button
                            variant="secondary"
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'email',
                                except: [],
                                only: [],
                                isRequired: false,
                                isUnique: false,
                              });
                            }}
                          >
                            <ToggleRight className="h-6 w-6" />
                            Email
                          </Button>
                        </div>
                        <div className="col-span-2 h-14">
                          <Button
                            variant="secondary"
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'url',
                                except: [],
                                only: [],
                                isRequired: false,
                                isUnique: false,
                              });
                            }}
                          >
                            <Link2 className="h-6 w-6" />
                            Url
                          </Button>
                        </div>
                        <div className="col-span-3 h-14">
                          <Button
                            variant="secondary"
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'select',
                                options: [],
                                maxSelected: null,
                                minSelected: null,
                                isRequired: false,
                                isUnique: false,
                              });
                            }}
                          >
                            <List className="h-6 w-6" />
                            Select
                          </Button>
                        </div>
                        <div className="col-span-3 h-14">
                          <Button
                            variant="secondary"
                            disabled={
                              tables.length === 0 ||
                              (tables.length === 1 && !!table)
                            }
                            sharedClasses="h-full w-full flex flex-col items-center justify-center"
                            onClick={() => {
                              setIsOpenPopover(false);
                              append({
                                name: lowestId(),
                                type: 'relation',
                                target: 'single',
                                table: '',
                                cascadeDelete: false,
                                maxSelected: null,
                                minSelected: null,
                                isRequired: false,
                                isUnique: false,
                              });
                            }}
                          >
                            <Database className="h-6 w-6" />
                            Relation
                          </Button>
                        </div>
                      </div>
                    </PopoverContent>
                  </Popover>
                </TabsContent>
                <TabsContent value="permissions" className="space-y-2">
                  {(['view', 'create', 'update', 'delete'] as const).map(
                    (permission) => (
                      <FormField
                        key={permission}
                        control={form.control}
                        name={`permissions.${permission}`}
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>{title(permission)}</FormLabel>
                            <FormControl>
                              <Input
                                {...field}
                                onlyDisableInput
                                data-form-type="other"
                                value={field.value ?? ''}
                                disabled={field.value === null}
                                placeholder={
                                  field.value === null
                                    ? 'Admin only'
                                    : 'Leave empty for no restrictions'
                                }
                                endAdornment={
                                  <TooltipProvider>
                                    <Tooltip>
                                      <TooltipTrigger asChild>
                                        <button
                                          type="button"
                                          onClick={() =>
                                            form.setValue(
                                              `permissions.${permission}`,
                                              field.value === null ? '' : null,
                                            )
                                          }
                                        >
                                          {field.value === null ? (
                                            <LockOpen className="h-4 w-4" />
                                          ) : (
                                            <Lock className="h-4 w-4" />
                                          )}
                                        </button>
                                      </TooltipTrigger>
                                      <TooltipContent>
                                        {field.value === null ? (
                                          <p>Use custom rule</p>
                                        ) : (
                                          <p>Restrict to admin only</p>
                                        )}
                                      </TooltipContent>
                                    </Tooltip>
                                  </TooltipProvider>
                                }
                              />
                            </FormControl>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    ),
                  )}
                </TabsContent>
              </Tabs>
            </div>

            <SheetFooter className={cn(table && 'sm:justify-between')}>
              {table && (
                <Button
                  type="button"
                  variant="destructive"
                  onClick={() =>
                    void (async () => {
                      await deleteMutateAsync(table.name);

                      setIsOpen(false);

                      const tables = await queryClient.fetchQuery(
                        tablesQueryOptions(client),
                      );

                      if (tables.length === 0)
                        await navigate({
                          to: '/dashboard/projects/$projectId/tables',
                          params: { projectId: params.projectId },
                        });
                      else
                        await navigate({
                          to: '/dashboard/projects/$projectId/tables/$tableId',
                          params: {
                            projectId: params.projectId,
                            tableId: tables?.[0].name,
                          },
                        });
                    })()
                  }
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  Delete
                </Button>
              )}

              <div className="space-x-1">
                <SheetClose asChild>
                  <Button type="button" variant="ghost">
                    Cancel
                  </Button>
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

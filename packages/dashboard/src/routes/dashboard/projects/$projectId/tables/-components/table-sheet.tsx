import { Client, CustomTable, Field } from '@adrastos/lib';
import { PopoverContent, PopoverTrigger } from '@radix-ui/react-popover';
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
import { title } from 'radash';
import { useState } from 'react';

import {
  Button,
  Input,
  Label,
  Popover,
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
} from '~/hooks';
import { cn, mkId } from '~/lib';

export const TableSheet: React.FC<{
  client: Client;
  table?: CustomTable;
  className?: string;
}> = ({ client, table, className }) => {
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const params = useParams({
    from: '/dashboard/projects/$projectId',
  });

  const [name, setName] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [isOpenPopover, setIsOpenPopover] = useState(false);
  const [fields, setFields] = useState<
    (Field & {
      id: string;
    })[]
  >([]);
  const [permissions, setPermissions] = useState<{
    view: string | null;
    create: string | null;
    update: string | null;
    delete: string | null;
  }>({
    view: null,
    create: null,
    update: null,
    delete: null,
  });

  const { mutateAsync: createMutateAsync } = useCreateTableMutation();
  const { mutateAsync: deleteMutateAsync } = useDeleteTableMutation();

  return (
    <Sheet
      open={isOpen}
      onOpenChange={() => {
        setIsOpen((o) => !o);
        setName('');
        setFields([]);
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
      <SheetContent className="flex w-[500px] flex-col justify-between lg:max-w-[500px]">
        <div className="h-full">
          <SheetHeader className="mb-5">
            <SheetTitle>Create A New Table</SheetTitle>
          </SheetHeader>

          <div className="mb-3">
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              placeholder="Name"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          <Tabs defaultValue="fields">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="fields">Fields</TabsTrigger>
              <TabsTrigger value="permissions">Permissions</TabsTrigger>
            </TabsList>
            <TabsContent value="fields">
              {fields.length > 0 && (
                <div className="mb-3 max-h-[calc(100vh-330px)] space-y-3 overflow-auto">
                  {fields.map((f, index) => {
                    let field: React.ReactNode = null;
                    switch (f.type) {
                      case 'string':
                        field = (
                          <div className="mt-2 grid grid-cols-2 gap-2">
                            <div>
                              <Label htmlFor={mkId(f.id, 'name')}>Name</Label>
                              <Input
                                value={f.name}
                                id={mkId(f.id, 'name')}
                                placeholder="Name"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            name: e.target.value,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <Label htmlFor={mkId(f.id, 'pattern')}>
                                Pattern
                              </Label>
                              <Input
                                id={mkId(f.id, 'pattern')}
                                placeholder="Pattern"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            pattern: e.target.value,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <Label htmlFor={mkId(f.id, 'minLength')}>
                                Min Length
                              </Label>
                              <Input
                                type="number"
                                id={mkId(f.id, 'minLength')}
                                placeholder="Min Length"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            minLength: !isNaN(
                                              e.target.valueAsNumber,
                                            )
                                              ? e.target.valueAsNumber
                                              : null,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <Label htmlFor={mkId(f.id, 'maxLength')}>
                                Max Length
                              </Label>
                              <Input
                                type="number"
                                id={mkId(f.id, 'maxLength')}
                                placeholder="Max Length"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            maxLength: !isNaN(
                                              e.target.valueAsNumber,
                                            )
                                              ? e.target.valueAsNumber
                                              : null,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <div className="mt-2 flex flex-row space-x-5">
                                <div className="flex items-center space-x-2">
                                  <Switch
                                    size="sm"
                                    checked={f.isRequired}
                                    id={mkId(f.id, 'isRequired')}
                                    onCheckedChange={() =>
                                      setFields((fields) =>
                                        fields.map((field) =>
                                          field.id === f.id &&
                                          field.type !== 'boolean'
                                            ? {
                                                ...field,
                                                isRequired: !field.isRequired,
                                              }
                                            : field,
                                        ),
                                      )
                                    }
                                  />
                                  <Label htmlFor={mkId(f.id, 'isRequired')}>
                                    Required
                                  </Label>
                                </div>

                                <div className="flex items-center space-x-2">
                                  <Switch
                                    size="sm"
                                    checked={f.isUnique}
                                    id={mkId(f.id, 'isUnique')}
                                    onCheckedChange={() =>
                                      setFields((fields) =>
                                        fields.map((field) =>
                                          field.id === f.id &&
                                          field.type !== 'boolean'
                                            ? {
                                                ...field,
                                                isUnique: !field.isUnique,
                                              }
                                            : field,
                                        ),
                                      )
                                    }
                                  />
                                  <Label htmlFor={mkId(f.id, 'isUnique')}>
                                    Unique
                                  </Label>
                                </div>
                              </div>
                            </div>
                          </div>
                        );
                        break;
                      case 'number':
                        field = (
                          <div className="mt-2 grid grid-cols-2 gap-2">
                            <div className="col-span-2">
                              <Label htmlFor={mkId(f.id, 'name')}>Name</Label>
                              <Input
                                value={f.name}
                                id={mkId(f.id, 'name')}
                                placeholder="Name"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            name: e.target.value,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <Label htmlFor={mkId(f.id, 'min')}>Min</Label>
                              <Input
                                type="number"
                                id={mkId(f.id, 'min')}
                                placeholder="Min"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            min: !isNaN(e.target.valueAsNumber)
                                              ? e.target.valueAsNumber
                                              : null,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <Label htmlFor={mkId(f.id, 'max')}>Max</Label>
                              <Input
                                type="number"
                                id={mkId(f.id, 'max')}
                                placeholder="Max"
                                onChange={(e) =>
                                  setFields((fields) =>
                                    fields.map((field) =>
                                      field.id === f.id
                                        ? {
                                            ...field,
                                            max: !isNaN(e.target.valueAsNumber)
                                              ? e.target.valueAsNumber
                                              : null,
                                          }
                                        : field,
                                    ),
                                  )
                                }
                              />
                            </div>

                            <div>
                              <div className="mt-2 flex flex-row space-x-5">
                                <div className="flex items-center space-x-2">
                                  <Switch
                                    size="sm"
                                    checked={f.isRequired}
                                    id={mkId(f.id, 'isRequired')}
                                    onCheckedChange={() =>
                                      setFields((fields) =>
                                        fields.map((field) =>
                                          field.id === f.id &&
                                          field.type !== 'boolean'
                                            ? {
                                                ...field,
                                                isRequired: !field.isRequired,
                                              }
                                            : field,
                                        ),
                                      )
                                    }
                                  />
                                  <Label htmlFor={mkId(f.id, 'isRequired')}>
                                    Required
                                  </Label>
                                </div>

                                <div className="flex items-center space-x-2">
                                  <Switch
                                    size="sm"
                                    checked={f.isUnique}
                                    id={mkId(f.id, 'isUnique')}
                                    onCheckedChange={() =>
                                      setFields((fields) =>
                                        fields.map((field) =>
                                          field.id === f.id &&
                                          field.type !== 'boolean'
                                            ? {
                                                ...field,
                                                isUnique: !field.isUnique,
                                              }
                                            : field,
                                        ),
                                      )
                                    }
                                  />
                                  <Label htmlFor={mkId(f.id, 'isUnique')}>
                                    Unique
                                  </Label>
                                </div>
                              </div>
                            </div>
                          </div>
                        );
                        break;
                      default:
                    }

                    return (
                      <div key={index} className="rounded-md border p-3 pt-2">
                        <div className="flex flex-row items-center justify-between">
                          <h3 className="text-base font-medium">
                            {title(f.type)} Field
                          </h3>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() =>
                              setFields((fields) =>
                                fields.filter((field) => field.id !== f.id),
                              )
                            }
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
                  <div className="grid grid-cols-6 gap-2 rounded-md border p-3">
                    <div className="col-span-2 h-14">
                      <Button
                        variant="secondary"
                        onClick={() => {
                          setIsOpenPopover(false);
                          setFields([
                            ...fields,
                            {
                              id: Math.random().toString(36).substring(2),
                              name: `field${
                                fields.length === 0 ? '' : fields.length
                              }`,
                              type: 'string',
                              isRequired: false,
                              isUnique: false,
                              maxLength: null,
                              minLength: null,
                              pattern: null,
                            },
                          ]);
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
                          setFields([
                            ...fields,
                            {
                              id: Math.random().toString(36).substring(2),
                              name: `field${
                                fields.length === 0 ? '' : fields.length
                              }`,
                              type: 'number',
                              isRequired: false,
                              isUnique: false,
                              max: null,
                              min: null,
                            },
                          ]);
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
                      >
                        <ToggleRight className="h-6 w-6" />
                        Boolean
                      </Button>
                    </div>
                    <div className="col-span-2 h-14">
                      <Button
                        variant="secondary"
                        sharedClasses="h-full w-full flex flex-col items-center justify-center"
                      >
                        <Calendar className="h-6 w-6" />
                        Date
                      </Button>
                    </div>
                    <div className="col-span-2 h-14">
                      <Button
                        variant="secondary"
                        sharedClasses="h-full w-full flex flex-col items-center justify-center"
                      >
                        <ToggleRight className="h-6 w-6" />
                        Email
                      </Button>
                    </div>
                    <div className="col-span-2 h-14">
                      <Button
                        variant="secondary"
                        sharedClasses="h-full w-full flex flex-col items-center justify-center"
                      >
                        <Link2 className="h-6 w-6" />
                        Url
                      </Button>
                    </div>
                    <div className="col-span-3 h-14">
                      <Button
                        variant="secondary"
                        sharedClasses="h-full w-full flex flex-col items-center justify-center"
                      >
                        <List className="h-6 w-6" />
                        Select
                      </Button>
                    </div>
                    <div className="col-span-3 h-14">
                      <Button
                        variant="secondary"
                        sharedClasses="h-full w-full flex flex-col items-center justify-center"
                      >
                        <Database className="h-6 w-6" />
                        Relation
                      </Button>
                    </div>
                  </div>
                </PopoverContent>
              </Popover>
            </TabsContent>
            <TabsContent value="permissions">
              <Label htmlFor="view">View</Label>
              <Input
                id="view"
                onlyDisableInput
                disabled={permissions.view === null}
                placeholder={
                  permissions.view === null
                    ? 'Admin only'
                    : 'Leave empty for no restrictions'
                }
                value={permissions.view ?? ''}
                onChange={(e) =>
                  setPermissions((p) => ({ ...p, view: e.target.value }))
                }
                endAdornment={
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <button
                          type="button"
                          onClick={() =>
                            setPermissions((p) => ({
                              ...p,
                              view: p.view === null ? '' : null,
                            }))
                          }
                        >
                          {permissions.view === null ? (
                            <LockOpen className="h-4 w-4" />
                          ) : (
                            <Lock className="h-4 w-4" />
                          )}
                        </button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {permissions.view === null ? (
                          <p>Use custom rule</p>
                        ) : (
                          <p>Restrict to admin only</p>
                        )}
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                }
              />

              <Label htmlFor="create">Create</Label>
              <Input
                id="create"
                onlyDisableInput
                disabled={permissions.create === null}
                placeholder={
                  permissions.create === null
                    ? 'Admin only'
                    : 'Leave empty for no restrictions'
                }
                value={permissions.create ?? ''}
                onChange={(e) =>
                  setPermissions((p) => ({ ...p, create: e.target.value }))
                }
                endAdornment={
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <button
                          type="button"
                          onClick={() =>
                            setPermissions((p) => ({
                              ...p,
                              create: p.create === null ? '' : null,
                            }))
                          }
                        >
                          {permissions.create === null ? (
                            <LockOpen className="h-4 w-4" />
                          ) : (
                            <Lock className="h-4 w-4" />
                          )}
                        </button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {permissions.create === null ? (
                          <p>Use custom rule</p>
                        ) : (
                          <p>Restrict to admin only</p>
                        )}
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                }
              />

              <Label htmlFor="update">Update</Label>
              <Input
                id="update"
                onlyDisableInput
                disabled={permissions.update === null}
                placeholder={
                  permissions.update === null
                    ? 'Admin only'
                    : 'Leave empty for no restrictions'
                }
                value={permissions.update ?? ''}
                onChange={(e) =>
                  setPermissions((p) => ({ ...p, update: e.target.value }))
                }
                endAdornment={
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <button
                          type="button"
                          onClick={() =>
                            setPermissions((p) => ({
                              ...p,
                              update: p.update === null ? '' : null,
                            }))
                          }
                        >
                          {permissions.update === null ? (
                            <LockOpen className="h-4 w-4" />
                          ) : (
                            <Lock className="h-4 w-4" />
                          )}
                        </button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {permissions.update === null ? (
                          <p>Use custom rule</p>
                        ) : (
                          <p>Restrict to admin only</p>
                        )}
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                }
              />

              <Label htmlFor="delete">Delete</Label>
              <Input
                id="delete"
                onlyDisableInput
                disabled={permissions.delete === null}
                placeholder={
                  permissions.delete === null
                    ? 'Admin only'
                    : 'Leave empty for no restrictions'
                }
                value={permissions.delete ?? ''}
                onChange={(e) =>
                  setPermissions((p) => ({ ...p, delete: e.target.value }))
                }
                endAdornment={
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <button
                          type="button"
                          onClick={() =>
                            setPermissions((p) => ({
                              ...p,
                              delete: p.delete === null ? '' : null,
                            }))
                          }
                        >
                          {permissions.delete === null ? (
                            <LockOpen className="h-4 w-4" />
                          ) : (
                            <Lock className="h-4 w-4" />
                          )}
                        </button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {permissions.delete === null ? (
                          <p>Use custom rule</p>
                        ) : (
                          <p>Restrict to admin only</p>
                        )}
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                }
              />
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
                  setIsOpenPopover(false);
                  setName('');
                  setFields([]);

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
              <Button variant="ghost">Cancel</Button>
            </SheetClose>
            <Button
              onClick={() => {
                void (async () => {
                  const table = await createMutateAsync({
                    name,
                    fields,
                    permissions,
                  });

                  setIsOpen(false);
                  setIsOpenPopover(false);
                  setName('');
                  setFields([]);

                  await navigate({
                    to: '/dashboard/projects/$projectId/tables/$tableId',
                    params: {
                      projectId: params.projectId,
                      tableId: table.name,
                    },
                  });
                })();
              }}
            >
              Submit
            </Button>
          </div>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};

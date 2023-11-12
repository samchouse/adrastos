import { table, TFWithModifiers } from '@adrastos/lib';
import { PopoverContent, PopoverTrigger } from '@radix-ui/react-popover';
import {
  Calendar,
  Database,
  Hash,
  Link2,
  List,
  Plus,
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
} from '~/components';
import { useCreateTableMutation } from '~/hooks';
import { Field, mkId } from '~/lib';

export const TableSheet: React.FC = () => {
  const [name, setName] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [fields, setFields] = useState<
    (Field & {
      id: string;
    })[]
  >([]);

  const { mutate } = useCreateTableMutation();

  return (
    <Sheet
      onOpenChange={() => {
        setName('');
        setFields([]);
      }}
    >
      <SheetTrigger asChild>
        <Button className="mb-3 w-full">
          <Plus className="mr-2 h-4 w-4" /> Create New
        </Button>
      </SheetTrigger>
      <SheetContent className="flex w-[500px] flex-col justify-between lg:max-w-[500px]">
        <div className="h-full">
          <SheetHeader className="mb-5">
            <SheetTitle>Create A New Table</SheetTitle>
          </SheetHeader>

          <div className="flex flex-col gap-y-5">
            <div>
              <Label htmlFor="name">Name</Label>
              <Input
                id="name"
                placeholder="Name"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>

            {fields.length > 0 && (
              <div className="max-h-[calc(100vh-300px)] space-y-3 overflow-auto">
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
                                      ? { ...field, name: e.target.value }
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
                                      ? { ...field, pattern: e.target.value }
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
                            <div className="mt-2 flex flex-row space-x-5">
                              <div className="flex items-center space-x-2">
                                <Switch
                                  size="sm"
                                  checked={f.isRequired}
                                  id={mkId(f.id, 'isRequired')}
                                  onCheckedChange={() =>
                                    setFields((fields) =>
                                      fields.map((field) =>
                                        field.id === f.id
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
                                        field.id === f.id
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
                                      ? { ...field, name: e.target.value }
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
                            <div className="mt-2 flex flex-row space-x-5">
                              <div className="flex items-center space-x-2">
                                <Switch
                                  size="sm"
                                  checked={f.isRequired}
                                  id={mkId(f.id, 'isRequired')}
                                  onCheckedChange={() =>
                                    setFields((fields) =>
                                      fields.map((field) =>
                                        field.id === f.id
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
                                        field.id === f.id
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

            <Popover open={isOpen} onOpenChange={() => setIsOpen((o) => !o)}>
              <PopoverTrigger asChild>
                <Button className="w-full" variant="secondary">
                  Add field
                </Button>
              </PopoverTrigger>
              <PopoverContent
                className="bg-background w-[451px]"
                sideOffset={8}
              >
                <div className="grid grid-cols-6 gap-2 rounded-md border p-3">
                  <div className="col-span-2 h-14">
                    <Button
                      variant="secondary"
                      onClick={() => {
                        setIsOpen(false);
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
                        setIsOpen(false);
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
          </div>
        </div>

        <SheetFooter>
          <SheetClose asChild>
            <Button variant="ghost">Cancel</Button>
          </SheetClose>
          <Button
            onClick={() => {
              mutate(
                table(name, (b) =>
                  fields.reduce(
                    (acc, curr) => {
                      switch (curr.type) {
                        case 'string': {
                          let field = b.string();
                          if (curr.maxLength)
                            field = field.maxLength(curr.maxLength);
                          if (curr.minLength)
                            field = field.minLength(curr.minLength);

                          acc[curr.name] = field;
                          break;
                        }
                        case 'number': {
                          let field = b.number();
                          if (curr.max) field = field.max(curr.max);
                          if (curr.min) field = field.min(curr.min);

                          acc[curr.name] = field;
                          break;
                        }
                        default:
                          break;
                      }

                      let field = acc[curr.name];

                      if (field.type === 'boolean') return acc;
                      if (!curr.isRequired)
                        field = field.optional() as TFWithModifiers;

                      if (field.type === 'boolean') return acc;
                      if (curr.isUnique)
                        field = field.unique() as TFWithModifiers;

                      acc[curr.name] = field;
                      return acc;
                    },
                    {} as Record<string, TFWithModifiers>,
                  ),
                ),
              );

              setIsOpen(false);
              setName('');
              setFields([]);
            }}
          >
            Submit
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
};

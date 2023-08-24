import { PopoverContent, PopoverTrigger } from '@radix-ui/react-popover';
import {
  Calendar,
  Database,
  Hash,
  Link2,
  List,
  Plus,
  ToggleRight,
  Type
} from 'lucide-react';

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
  SheetTrigger
} from '~/components';

export const CreateSheet: React.FC = () => (
  <Sheet>
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

        <div className="flex h-full flex-col gap-y-5">
          <div>
            <Label htmlFor="name">Name</Label>
            <Input id="name" placeholder="Name"></Input>
          </div>

          <Popover>
            <PopoverTrigger asChild>
              <Button className="w-full" variant="secondary">
                Add field
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-[451px]" sideOffset={8}>
              <div className="grid grid-cols-6 gap-2 rounded-md border p-3">
                <div className="bg-secondary col-span-2 flex h-14 flex-col items-center justify-center rounded-md">
                  <Type className="h-6 w-6" />
                  String
                </div>
                <div className="bg-secondary col-span-2 flex h-14 flex-col items-center justify-center rounded-md">
                  <Hash className="h-6 w-6" />
                  Number
                </div>
                <div className="bg-secondary col-span-2 flex h-14 flex-col items-center justify-center rounded-md">
                  <ToggleRight className="h-6 w-6" />
                  Boolean
                </div>
                <div className="bg-secondary col-span-2 flex h-14 flex-col items-center justify-center rounded-md">
                  <Calendar className="h-6 w-6" />
                  Date
                </div>
                <div className="bg-secondary col-span-2 flex h-14 flex-col items-center justify-center rounded-md">
                  <ToggleRight className="h-6 w-6" />
                  Email
                </div>
                <div className="bg-secondary col-span-2 flex h-14 flex-col items-center justify-center rounded-md">
                  <Link2 className="h-6 w-6" />
                  Url
                </div>
                <div className="bg-secondary col-span-3 flex h-14 flex-col items-center justify-center rounded-md">
                  <List className="h-6 w-6" />
                  Select
                </div>
                <div className="bg-secondary col-span-3 flex h-14 flex-col items-center justify-center rounded-md">
                  <Database className="h-6 w-6" />
                  Relation
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
        <Button>Submit</Button>
      </SheetFooter>
    </SheetContent>
  </Sheet>
);

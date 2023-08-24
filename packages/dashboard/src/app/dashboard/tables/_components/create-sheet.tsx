import { Plus } from 'lucide-react';

import {
  Button,
  Input,
  Label,
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

        <div className="h-full">
          <Label htmlFor="name">Name</Label>
          <Input id="name" placeholder="Name"></Input>
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

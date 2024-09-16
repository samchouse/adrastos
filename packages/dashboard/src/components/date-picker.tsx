import { format } from 'date-fns';
import { CalendarIcon } from 'lucide-react';
import type { SelectSingleEventHandler } from 'react-day-picker';

import { cn } from '~/lib';

import {
  Button,
  Calendar,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from './ui';

export const DatePicker: React.FC<{
  value?: Date;
  onChange: SelectSingleEventHandler;
}> = ({ value, onChange }) => (
  <Popover>
    <PopoverTrigger asChild>
      <Button
        variant="outline"
        className={cn(
          'w-[290px] justify-start text-left font-normal',
          !value && 'text-muted-foreground',
        )}
      >
        <CalendarIcon className="mr-2 size-4" />
        {value ? format(value, 'PPP') : <span>Pick a date</span>}
      </Button>
    </PopoverTrigger>
    <PopoverContent className="w-[290px] p-0">
      <Calendar
        initialFocus
        mode="single"
        selected={value}
        onSelect={onChange}
        captionLayout="dropdown-buttons"
        toYear={new Date().getFullYear() + 20}
        fromYear={new Date().getFullYear() - 50}
      />
    </PopoverContent>
  </Popover>
);

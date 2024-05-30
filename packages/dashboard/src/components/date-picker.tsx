import { format } from 'date-fns';
import { CalendarIcon } from 'lucide-react';
import { SelectSingleEventHandler } from 'react-day-picker';

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
        <CalendarIcon className="mr-2 h-4 w-4" />
        {value ? format(value, 'PPP') : <span>Pick a date</span>}
      </Button>
    </PopoverTrigger>
    <PopoverContent className="w-[290px] p-0">
      <Calendar
        mode="single"
        fromYear={new Date().getFullYear() - 50}
        toYear={new Date().getFullYear() + 20}
        selected={value}
        onSelect={onChange}
        initialFocus
        captionLayout="dropdown-buttons"
      />
    </PopoverContent>
  </Popover>
);

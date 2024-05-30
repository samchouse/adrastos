import { Command as CommandPrimitive } from 'cmdk';
import { X } from 'lucide-react';
import * as React from 'react';

import { Badge, Command, CommandGroup, CommandItem, CommandList } from './ui';

export interface Option {
  value: string;
  label: string;
}

export const MultiSelect = React.forwardRef<
  HTMLInputElement,
  {
    options: Option[];
    selected?: Option[];
    onSelectedChange?: (values: Option[]) => void;
  } & Omit<React.InputHTMLAttributes<HTMLInputElement>, 'value' | 'onChange'>
>(
  (
    { options, selected: controlledSelected, onSelectedChange, ...props },
    ref,
  ) => {
    const inputRef = React.useRef<HTMLInputElement>(null);
    const [open, setOpen] = React.useState(false);
    const [selected, setSelected] = React.useState<Option[]>(
      controlledSelected ?? [],
    );
    const [inputValue, setInputValue] = React.useState('');

    React.useImperativeHandle(ref, () => inputRef.current!);

    const handleUnselect = React.useCallback((option: Option) => {
      setSelected((prev) => prev.filter((s) => s.value !== option.value));
    }, []);

    const handleKeyDown = React.useCallback(
      (e: React.KeyboardEvent<HTMLDivElement>) => {
        const input = inputRef.current;
        if (input) {
          if (e.key === 'Delete' || e.key === 'Backspace') {
            if (input.value === '') {
              setSelected((prev) => {
                const newSelected = [...prev];
                newSelected.pop();
                return newSelected;
              });
            }
          }
          // This is not a default behaviour of the <input /> field
          if (e.key === 'Escape') {
            input.blur();
          }
        }
      },
      [],
    );

    const [previousSelected, setPreviousSelected] = React.useState(selected);
    React.useEffect(() => {
      if (
        !selected.every(
          (opt) => !!previousSelected.find((o) => o.value === opt.value),
        ) ||
        !previousSelected.every(
          (opt) => !!selected.find((o) => o.value === opt.value),
        )
      ) {
        setPreviousSelected(selected);
        onSelectedChange?.(selected);
      }
    }, [onSelectedChange, previousSelected, selected]);

    const selectable = options.filter(
      (option) => !selected.some((s) => s.value === option.value),
    );

    return (
      <Command
        onKeyDown={handleKeyDown}
        className="overflow-visible bg-transparent"
      >
        <div className="group min-h-10 rounded-md border border-input px-3 py-2 text-sm ring-offset-background focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2">
          <div className="flex flex-wrap gap-1">
            {selected.map((option) => (
              <Badge key={option.value} variant="secondary">
                {option.label}
                <button
                  className="ml-1 rounded-full outline-none ring-offset-background focus:ring-2 focus:ring-ring focus:ring-offset-2"
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleUnselect(option);
                    }
                  }}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                  }}
                  onClick={() => handleUnselect(option)}
                >
                  <X className="h-3 w-3 text-muted-foreground hover:text-foreground" />
                </button>
              </Badge>
            ))}
            {/* Avoid having the "Search" Icon */}
            <CommandPrimitive.Input
              {...props}
              ref={inputRef}
              value={inputValue}
              onValueChange={setInputValue}
              className="flex-1 bg-transparent outline-none placeholder:text-foreground"
              onBlur={(e) => {
                setOpen(false);
                props.onBlur?.(e);
              }}
              onFocus={(e) => {
                setOpen(true);
                props.onFocus?.(e);
              }}
            />
          </div>
        </div>
        {open && selectable.length > 0 ? (
          <div className="relative mt-2">
            <div className="absolute top-0 z-10 w-full rounded-md border bg-popover text-popover-foreground shadow-md outline-none animate-in">
              <CommandList>
                <CommandGroup className="h-full overflow-auto">
                  {selectable.map((option) => (
                    <CommandItem
                      key={option.value}
                      onMouseDown={(e) => {
                        e.preventDefault();
                        e.stopPropagation();
                      }}
                      onSelect={() => {
                        setInputValue('');
                        setSelected((prev) => [...prev, option]);
                      }}
                      className={'cursor-pointer'}
                    >
                      {option.label}
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </div>
          </div>
        ) : null}
      </Command>
    );
  },
);

import * as React from 'react';

import { cn } from '~/lib/utils';

export interface InputProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  endAdornment?: JSX.Element;
  onlyDisableInput?: boolean;
  startAdornment?: JSX.Element;
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  (
    {
      className,
      type,
      startAdornment,
      endAdornment,
      onlyDisableInput,
      ...props
    },
    ref,
  ) => {
    const hasAdornment = Boolean(startAdornment) || Boolean(endAdornment);
    const disabled = props.disabled
      ? onlyDisableInput
        ? 'partial'
        : true
      : false;

    return (
      <>
        {hasAdornment ? (
          <div
            data-disabled={disabled}
            className="flex h-10 items-center justify-center gap-2 rounded-md border border-input bg-transparent px-3 ring-offset-background focus-within:ring-1 focus-within:ring-ring focus-within:ring-offset-2 data-[disabled=partial]:cursor-not-allowed data-[disabled=true]:cursor-not-allowed data-[disabled=partial]:border-input/50 data-[disabled=true]:opacity-50"
          >
            {startAdornment && (
              <div
                className={cn(
                  'flex text-muted-foreground',
                  onlyDisableInput && 'cursor-default',
                )}
              >
                {startAdornment}
              </div>
            )}
            <input
              ref={ref}
              type={type}
              className={cn(
                'flex size-full rounded-md border-none bg-transparent py-2 text-sm shadow-none outline-none file:bg-transparent file:font-medium file:text-sm placeholder:text-muted-foreground focus-visible:border-none focus-visible:shadow-none focus-visible:outline-none disabled:cursor-not-allowed',
                disabled === 'partial' && 'disabled:opacity-50',
                className,
              )}
              {...props}
            />
            {endAdornment && (
              <div
                className={cn(
                  'flex text-muted-foreground',
                  onlyDisableInput && 'cursor-default',
                )}
              >
                {endAdornment}
              </div>
            )}
          </div>
        ) : (
          <input
            ref={ref}
            type={type}
            className={cn(
              'flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:font-medium file:text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50',
              className,
            )}
            {...props}
          />
        )}
      </>
    );
  },
);
Input.displayName = 'Input';

export { Input };

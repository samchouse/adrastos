import {
  isToday as _isToday,
  CalendarDate,
  createCalendar,
  fromDate,
  getLocalTimeZone,
  getWeeksInMonth,
  parseDateTime,
  toCalendarDate,
  toCalendarDateTime,
} from '@internationalized/date';
import { DateSegment as IDateSegment } from '@react-stately/datepicker';
import {
  CalendarIcon,
  ChevronDown,
  ChevronLeftIcon,
  ChevronRightIcon,
  X,
} from 'lucide-react';
import React, {
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from 'react';
import {
  AriaDatePickerProps,
  AriaDialogProps,
  AriaTimeFieldProps,
  CalendarProps,
  DateValue,
  TimeValue,
  useButton,
  useCalendar,
  useCalendarCell,
  useCalendarGrid,
  useDateField,
  useDatePicker,
  useDateSegment,
  useLocale,
  useTimeField,
} from 'react-aria';
import {
  CalendarState,
  DateFieldState,
  DatePickerState,
  DatePickerStateOptions,
  TimeFieldStateOptions,
  useCalendarState,
  useDateFieldState,
  useDatePickerState,
  useTimeFieldState,
} from 'react-stately';

import { cn } from '~/lib/utils';

import {
  Button,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Separator,
} from './ui';
import { buttonVariants } from './ui/button';

const months = [
  'January',
  'February',
  'March',
  'April',
  'May',
  'June',
  'July',
  'August',
  'September',
  'October',
  'November',
  'December',
];

// million-ignore
function Calendar({
  hasReset,
  setHasReset,
  jsDatetime,
  ...props
}: CalendarProps<DateValue> & {
  hasReset: boolean;
  jsDatetime: Date | null;
  setHasReset: React.Dispatch<React.SetStateAction<boolean>>;
}) {
  const prevButtonRef = React.useRef<HTMLButtonElement | null>(null);
  const nextButtonRef = React.useRef<HTMLButtonElement | null>(null);

  const years = useMemo(() => {
    const currentYear = new Date().getFullYear();
    return Array.from({ length: 71 }, (_, i) => currentYear - 50 + i);
  }, []);

  const { locale } = useLocale();
  const state = useCalendarState({
    ...props,
    locale,
    createCalendar,
  });
  const {
    calendarProps,
    prevButtonProps: _prevButtonProps,
    nextButtonProps: _nextButtonProps,
    title,
  } = useCalendar(props, state);
  const { buttonProps: prevButtonProps } = useButton(
    _prevButtonProps,
    prevButtonRef,
  );
  const { buttonProps: nextButtonProps } = useButton(
    _nextButtonProps,
    nextButtonRef,
  );

  const [month, year] = title.split(' ');

  useEffect(() => {
    if (jsDatetime === null && !hasReset) {
      setHasReset(true);

      const date = new Date();
      state.setFocusedDate(
        new CalendarDate(
          date.getFullYear(),
          date.getMonth() + 1,
          date.getDate(),
        ),
      );
    }
  }, [state, hasReset, setHasReset, jsDatetime]);

  return (
    <div {...calendarProps} className="flex flex-col items-center space-y-3">
      <div className="relative flex w-full items-center justify-center">
        <Button
          {...prevButtonProps}
          ref={prevButtonRef}
          withSpan={false}
          variant="outline"
          className={cn(
            'absolute left-1 h-7 w-7 bg-transparent p-0 opacity-50 hover:opacity-100',
          )}
        >
          <ChevronLeftIcon className="h-4 w-4" />
        </Button>
        <div className="flex w-full flex-row px-9 font-medium">
          <div
            className={cn(
              buttonVariants({ variant: 'ghost', size: 'xs' }),
              'relative inline-flex flex-1 items-center text-base font-normal',
            )}
          >
            <select
              name="months"
              aria-label="Month: "
              className="absolute inset-0 z-10 w-full cursor-pointer appearance-none opacity-0"
              onChange={(e) =>
                state.setFocusedDate(
                  new CalendarDate(
                    parseInt(year, 10),
                    parseInt(e.target.value, 10) + 1,
                    1,
                  ),
                )
              }
            >
              {months.map((month, i) => (
                <option key={i} value={i}>
                  {month}
                </option>
              ))}
            </select>
            <div aria-hidden="true" className="flex items-center text-sm">
              {month}
              <ChevronDown className="ml-1 mt-[2px] h-4 w-4" />
            </div>
          </div>
          <div
            className={cn(
              buttonVariants({ variant: 'ghost', size: 'xs' }),
              'relative inline-flex items-center text-base font-normal',
            )}
          >
            <select
              name="years"
              aria-label="Year: "
              className="absolute inset-0 z-10 w-full cursor-pointer appearance-none opacity-0"
              onChange={(e) =>
                state.setFocusedDate(
                  new CalendarDate(
                    parseInt(e.target.value, 10),
                    months.findIndex(
                      (m) => m.toLowerCase() === month.toLowerCase(),
                    ) + 1,
                    1,
                  ),
                )
              }
            >
              {years.map((year) => (
                <option key={year} value={year}>
                  {year}
                </option>
              ))}
            </select>
            <div aria-hidden="true" className="flex items-center text-sm">
              {year}
              <ChevronDown className="ml-1 mt-[2px] h-4 w-4" />
            </div>
          </div>
        </div>
        <Button
          withSpan={false}
          {...nextButtonProps}
          ref={nextButtonRef}
          variant="outline"
          className={cn(
            'absolute right-1 h-7 w-7 bg-transparent p-0 opacity-50 hover:opacity-100',
          )}
        >
          <ChevronRightIcon className="h-4 w-4" />
        </Button>
      </div>
      <CalendarGrid state={state} />
    </div>
  );
}

interface CalendarGridProps {
  state: CalendarState;
}

// million-ignore
function CalendarGrid({ state, ...props }: CalendarGridProps) {
  const { locale } = useLocale();
  const { gridProps, headerProps, weekDays } = useCalendarGrid(
    { ...props, weekdayStyle: 'short' },
    state,
  );

  // Get the number of weeks in the month so we can render the proper number of rows.
  const weeksInMonth = getWeeksInMonth(state.visibleRange.start, locale);

  return (
    <table
      {...gridProps}
      className={cn(gridProps.className, 'border-collapse space-y-1')}
    >
      <thead {...headerProps}>
        <tr className="flex">
          {weekDays.map((day, index) => (
            <th
              className="w-9 rounded-md text-[0.8rem] font-normal text-muted-foreground"
              key={index}
            >
              {day}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {[...new Array(weeksInMonth).keys()].map((weekIndex) => (
          <tr className="mt-2 flex w-full" key={weekIndex}>
            {state
              .getDatesInWeek(weekIndex)
              .map((date, i) =>
                date ? (
                  <CalendarCell key={i} state={state} date={date} />
                ) : (
                  <td key={i} />
                ),
              )}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

interface CalendarCellProps {
  state: CalendarState;
  date: CalendarDate;
}

// million-ignore
function CalendarCell({ state, date }: CalendarCellProps) {
  const ref = React.useRef<HTMLButtonElement | null>(null);
  const {
    cellProps,
    buttonProps,
    isSelected,
    isOutsideVisibleRange,
    isDisabled,
    formattedDate,
  } = useCalendarCell({ date }, state, ref);

  const isToday = useMemo(() => {
    const timezone = getLocalTimeZone();
    return _isToday(date, timezone);
  }, [date]);

  return (
    <td
      {...cellProps}
      className={cn(
        cellProps.className,
        'relative p-0 text-center text-sm focus-within:relative focus-within:z-20 [&:has([aria-selected])]:bg-accent first:[&:has([aria-selected])]:rounded-l-md last:[&:has([aria-selected])]:rounded-r-md',
      )}
    >
      <Button
        {...buttonProps}
        type="button"
        variant="ghost"
        ref={ref}
        className={cn(
          buttonProps.className,
          'h-9 w-9',
          isToday && 'bg-accent text-accent-foreground',
          isSelected &&
            'bg-primary text-primary-foreground hover:bg-primary hover:text-primary-foreground focus:bg-primary focus:text-primary-foreground',
          isOutsideVisibleRange && 'text-muted-foreground opacity-50',
          isDisabled && 'text-muted-foreground opacity-50',
        )}
      >
        {formattedDate}
      </Button>
    </td>
  );
}

interface DateSegmentProps {
  segment: IDateSegment;
  state: DateFieldState;
}

// million-ignore
function DateSegment({ segment, state }: DateSegmentProps) {
  const ref = useRef(null);

  const {
    segmentProps: { ...segmentProps },
  } = useDateSegment(segment, state, ref);

  return (
    <div
      {...segmentProps}
      ref={ref}
      data-form-type="other"
      className={cn(
        'focus:rounded-[2px] focus:bg-accent focus:text-accent-foreground focus:outline-none',
        segment.type !== 'literal' && 'px-[1px]',
        segment.isPlaceholder && 'text-muted-foreground',
      )}
    >
      {segment.text}
    </div>
  );
}

// million-ignore
function DateField({
  contentRef,
  mutState,
  buttonProps,
  dialogProps,
  calendarProps,
  jsDatetime,
  ...props
}: AriaDatePickerProps<DateValue> & {
  mutState: DatePickerState;
  dialogProps: AriaDialogProps;
  calendarProps: CalendarProps<DateValue>;
  buttonProps: React.ButtonHTMLAttributes<HTMLButtonElement>;
  contentRef: React.MutableRefObject<HTMLDivElement | null>;
  jsDatetime: Date | null;
}) {
  const ref = useRef<HTMLDivElement | null>(null);

  const { locale } = useLocale();
  const state = useDateFieldState({
    ...props,
    locale,
    createCalendar,
  });
  const { fieldProps } = useDateField(props, state, ref);

  const [hasReset, setHasReset] = useState(false);
  useEffect(() => {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    if (jsDatetime === null && !hasReset) mutState.setTimeValue(null);
    if (jsDatetime !== null) setHasReset(false);
  }, [jsDatetime, mutState, hasReset]);

  return (
    <div className="flex h-10 flex-row">
      <div>
        <Popover open={props.isOpen} onOpenChange={props.onOpenChange}>
          <PopoverTrigger asChild>
            <Button
              {...buttonProps}
              variant="ghost"
              size="sm"
              className="h-full rounded-r-none"
              disabled={props.isDisabled}
              onClick={() => {
                mutState.setOpen(true);
              }}
            >
              <CalendarIcon className="h-5 w-5" />
            </Button>
          </PopoverTrigger>
          <PopoverContent ref={contentRef} className="w-[300px]">
            <div {...dialogProps} className="space-y-3">
              <Calendar
                {...calendarProps}
                hasReset={hasReset}
                setHasReset={setHasReset}
                jsDatetime={jsDatetime}
              />
              {mutState.hasTime && (
                <TimeField
                  value={mutState.timeValue}
                  onChange={(v) => mutState.setTimeValue(v)}
                />
              )}
            </div>
          </PopoverContent>
        </Popover>
      </div>

      <Separator orientation="vertical" />

      <div
        {...fieldProps}
        ref={ref}
        className={cn(
          'inline-flex h-10 flex-1 items-center rounded-l-md border-input bg-transparent p-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
          props.isDisabled && 'cursor-not-allowed opacity-50',
        )}
      >
        {state.segments.map((segment, i) => (
          <DateSegment key={i} segment={segment} state={state} />
        ))}
        {state.isInvalid && <span aria-hidden="true">🚫</span>}
      </div>
    </div>
  );
}

// million-ignore
function TimeField(props: AriaTimeFieldProps<TimeValue>) {
  const ref = useRef<HTMLDivElement | null>(null);

  const { locale } = useLocale();
  const state = useTimeFieldState({
    ...props,
    locale,
  });
  const { fieldProps } = useTimeField(props, state, ref);

  return (
    <div
      {...fieldProps}
      ref={ref}
      className={cn(
        'inline-flex h-10 w-full flex-1 rounded-md border border-input bg-transparent px-3 py-2 text-sm ring-offset-background focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        props.isDisabled && 'cursor-not-allowed opacity-50',
      )}
    >
      {state.segments.map((segment, i) => (
        <DateSegment key={i} segment={segment} state={state} />
      ))}
    </div>
  );
}

// million-ignore
const TimePicker = React.forwardRef<
  HTMLDivElement,
  Omit<TimeFieldStateOptions<TimeValue>, 'locale'>
>((props, _) => <TimeField {...props} />);

TimePicker.displayName = 'TimePicker';

export interface DateTimePickerRef {
  divRef: HTMLDivElement | null;
  buttonRef: HTMLButtonElement | null;
  contentRef: HTMLDivElement | null;
  jsDate: Date | null;
  state: DatePickerState;
}

// million-ignore
const DateTimePicker = React.forwardRef<
  DateTimePickerRef,
  DatePickerStateOptions<DateValue> & {
    jsDate?: Date | null;
    onJsDateChange?: (date: Date) => void;
    showClearButton?: boolean;
  }
>(({ jsDate, onJsDateChange, showClearButton = true, ...props }, ref) => {
  const divRef = useRef<HTMLDivElement | null>(null);
  const buttonRef = useRef<HTMLButtonElement | null>(null);
  const contentRef = useRef<HTMLDivElement | null>(null);
  const [jsDatetime, setJsDatetime] = useState(jsDate ?? null);

  const state = useDatePickerState(props);

  useImperativeHandle(ref, () => ({
    divRef: divRef.current,
    buttonRef: buttonRef.current,
    contentRef: contentRef.current,
    jsDate: jsDatetime,
    state,
  }));
  const {
    groupProps,
    fieldProps,
    buttonProps: _buttonProps,
    dialogProps,
    calendarProps,
  } = useDatePicker(props, state, divRef);
  const { buttonProps } = useButton(_buttonProps, buttonRef);

  const currentValue = useCallback(() => {
    if (!jsDatetime) {
      return null;
    }

    const parsed = fromDate(jsDatetime, getLocalTimeZone());

    if (state.hasTime) {
      return toCalendarDateTime(parsed);
    }

    return toCalendarDate(parsed);
  }, [jsDatetime, state.hasTime]);

  useEffect(() => {
    /**
     * If user types datetime, it will be a null value until we get the correct datetime.
     * This is controlled by react-aria.
     **/
    if (state.value) {
      const date = parseDateTime(state.value.toString()).toDate(
        getLocalTimeZone(),
      );
      setJsDatetime(date);
      onJsDateChange?.(date);
    }
  }, [state.value, onJsDateChange]);
  return (
    <div
      {...groupProps}
      ref={divRef}
      className={cn(
        groupProps.className,
        'flex h-10 items-center justify-between rounded-md border ring-offset-background focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2',
      )}
    >
      <DateField
        {...fieldProps}
        value={currentValue()}
        contentRef={contentRef}
        mutState={state}
        dialogProps={dialogProps}
        calendarProps={calendarProps}
        buttonProps={buttonProps}
        jsDatetime={jsDatetime}
      />
      <div className={cn('mr-2 h-5 w-5', !showClearButton && 'hidden')}>
        <X
          className={cn(
            'h-5 w-5 cursor-pointer text-muted-foreground',
            !jsDatetime && 'hidden',
          )}
          onClick={() => setJsDatetime(null)}
        />
      </div>
    </div>
  );
});

DateTimePicker.displayName = 'DateTimePicker';

export { DateTimePicker, TimePicker };

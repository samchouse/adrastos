import { Slot, type SlotProps } from '@radix-ui/react-slot';
import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
} from 'react';

type Maybe<T> = T | null | undefined;

const MultiDialogContainerContext = createContext<unknown>(null);
MultiDialogContainerContext.displayName = 'MultiDialogContainerContext';

export function useMultiDialog<T = unknown>(): [
  Maybe<T>,
  React.Dispatch<React.SetStateAction<Maybe<T>>>,
];
export function useMultiDialog<T = unknown>(
  v: T,
): [boolean, (v: boolean) => void];
// eslint-disable-next-line react-refresh/only-export-components
export function useMultiDialog<T = unknown>(v?: T) {
  const s = useContext(MultiDialogContainerContext) as [
    Maybe<T>,
    React.Dispatch<React.SetStateAction<Maybe<T>>>,
  ];
  if (!s)
    throw new Error(
      "Cannot use 'useMultiDialog' outside 'MultiDialogProvider'.",
    );
  if (v == null) return s;

  const [dialog, setDialog] = s;

  // eslint-disable-next-line react-hooks/rules-of-hooks
  const onOpenChange = useCallback(
    (o: boolean) => {
      o ? setDialog(v) : setDialog(null);
    },
    [setDialog, v],
  );

  const open = dialog === v;
  // eslint-disable-next-line react-hooks/rules-of-hooks
  const result = useMemo(
    () => [open, onOpenChange] as const,
    [onOpenChange, open],
  );

  return result;
}

export function MultiDialogTrigger<T = unknown>({
  value,
  onClick,
  ...props
}: SlotProps &
  React.RefAttributes<HTMLElement> & {
    value: T;
  }) {
  const [, open] = useMultiDialog(value);
  const oc = useCallback<React.MouseEventHandler<HTMLElement>>(
    (e) => {
      open(true);
      onClick?.(e);
    },
    [open, onClick],
  );
  return <Slot onClick={oc} {...props} />;
}

export function MultiDialogContainer<T = unknown>({
  value,
  children,
}: {
  value: T;
  children?: (options: {
    open: boolean;
    onOpenChange: (open: boolean) => void;
  }) => JSX.Element;
}) {
  const [open, onOpenChange] = useMultiDialog(value);

  return useMemo(
    () =>
      children
        ? children({
            open,
            onOpenChange,
          })
        : null,
    [children, onOpenChange, open],
  );
}

interface Builder<T = unknown> {
  readonly Trigger: (
    ...args: Parameters<typeof MultiDialogTrigger<T>>
  ) => React.ReactNode;
  readonly Container: (
    ...args: Parameters<typeof MultiDialogContainer<T>>
  ) => React.ReactNode;
}

const builder = {
  Trigger: MultiDialogTrigger,
  Container: MultiDialogContainer,
} as const;

export type MultiDialogBuilder<T = unknown> = (
  builder: Builder<T>,
) => React.ReactNode;
export function MultiDialog<T = unknown>({
  defaultOpen = null,
  children,
}: {
  defaultOpen?: T | null;
  children?: MultiDialogBuilder<T>;
}) {
  const [state, setState] = useState<T | null>(defaultOpen);

  return (
    <MultiDialogContainerContext.Provider value={[state, setState]}>
      {children?.(builder) ?? null}
    </MultiDialogContainerContext.Provider>
  );
}

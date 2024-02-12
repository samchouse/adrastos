import { RowData, SortingFn } from '@tanstack/react-table';

declare module '@tanstack/table-core' {
  interface ColumnMeta<TData extends RowData, TValue> {
    style?: {
      width?: 'min';
      textAlign?: 'left' | 'center' | 'right';
    };
  }
}

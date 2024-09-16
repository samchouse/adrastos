import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import {
  type ColumnDef,
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { format } from 'date-fns';
import { filesize } from 'filesize';
import {
  ChevronDown,
  ChevronUp,
  ExternalLink,
  File,
  Trash2,
} from 'lucide-react';
import { useCallback, useMemo, useState } from 'react';
import { useDropzone } from 'react-dropzone';

import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Skeleton,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from '~/components';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '~/components/ui/table';
import { storageQueryOptions, useDeleteUploadMutation } from '~/hooks';
import { cn } from '~/lib';
import type { Upload } from '~/types';

export const Route = createFileRoute('/dashboard/projects/$projectId/storage')({
  component: StorageComponent,
  loader: async ({ context: { client, queryClient } }) => {
    const list = await queryClient.ensureQueryData(storageQueryOptions(client));

    return { list };
  },
});

const columnHelper = createColumnHelper<Upload>();

interface DataTableProps<TData, TValue> {
  data: TData[];
  columns: ColumnDef<TData, TValue>[];
}

export function DataTable<TData, TValue>({
  columns,
  data,
}: DataTableProps<TData, TValue>) {
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    initialState: {
      sorting: [
        {
          id: 'createdAt',
          desc: true,
        },
      ],
      pagination: {
        pageSize: 15,
      },
    },
  });

  return (
    <div className="rounded-md border">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => (
                <TableHead
                  key={header.id}
                  className={cn(
                    header.column.columnDef.meta?.style?.width === 'min' &&
                      'w-[1%]',
                    header.column.columnDef.enableSorting && 'p-0',
                  )}
                >
                  {header.isPlaceholder ? null : (
                    <div className="w-min whitespace-nowrap">
                      {flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                    </div>
                  )}
                </TableHead>
              ))}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                className="cursor-pointer"
                data-state={row.getIsSelected() && 'selected'}
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell
                    key={cell.id}
                    className={cn(
                      'py-3',
                      cell.column.columnDef.meta?.style?.textAlign &&
                        `text-${cell.column.columnDef.meta.style.textAlign}`,
                      cell.column.columnDef.meta?.style?.width === 'min' &&
                        'w-min whitespace-nowrap',
                    )}
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

function StorageComponent() {
  const { client } = Route.useRouteContext();

  const [{ data: storage }] = useSuspenseQueries({
    queries: [storageQueryOptions(client)],
  });

  const { mutate } = useDeleteUploadMutation();

  const onDrop = useCallback((_acceptedFiles: File[]) => {
    // Do something with the files
  }, []);
  const { getRootProps, getInputProps, isDragActive, acceptedFiles } =
    useDropzone({ onDrop });

  const [loadedImages, setLoadedImages] = useState<string[]>([]);
  const columns = useMemo(
    () =>
      [
        columnHelper.display({
          id: 'preview',
          meta: {
            style: {
              width: 'min',
            },
          },
          cell: ({ row }) => (
            <a
              target="_blank"
              href={`${import.meta.env.VITE_BACKEND_URL ?? ''}/api/storage/get/${row.original.id}/${row.original.name}?projectId=${client.projectId}`}
              rel="noreferrer"
            >
              <div className="flex size-10 items-center justify-center">
                {row.original.type.startsWith('image/') ? (
                  <>
                    {!loadedImages.includes(row.original.id) && (
                      <Skeleton className="size-10 rounded-full" />
                    )}
                    <img
                      src={`${import.meta.env.VITE_BACKEND_URL ?? ''}/api/storage/get/${row.original.id}/${row.original.name}?projectId=${client.projectId}`}
                      alt="User upload"
                      onLoad={() =>
                        !loadedImages.includes(row.original.id) &&
                        setLoadedImages((i) => [...i, row.original.id])
                      }
                      className={cn(
                        'size-10 rounded-full object-cover',
                        !loadedImages.includes(row.original.id) && 'hidden',
                      )}
                    />
                  </>
                ) : (
                  <File className="size-8 text-muted-foreground" />
                )}
              </div>
            </a>
          ),
        }),
        columnHelper.accessor('name', { header: 'Name' }),
        columnHelper.accessor('type', { header: 'Content Type' }),
        columnHelper.accessor('size', {
          enableSorting: true,
          header: ({ column }) => (
            <Button
              variant="ghost"
              className="group"
              onClick={() => {
                column.toggleSorting(
                  column.getIsSorted() ? column.getIsSorted() === 'asc' : true,
                );
              }}
            >
              Size
              {column.getIsSorted() === 'asc' ? (
                <ChevronUp
                  className={cn(
                    'invisible ml-2 size-4 group-hover:visible',
                    column.getIsSorted() && 'visible',
                  )}
                />
              ) : (
                <ChevronDown
                  className={cn(
                    'invisible ml-2 size-4 group-hover:visible',
                    column.getIsSorted() && 'visible',
                  )}
                />
              )}
            </Button>
          ),
          cell: ({ getValue }) => filesize(getValue()),
        }),
        columnHelper.accessor('createdAt', {
          enableSorting: true,
          meta: {
            style: {
              width: 'min',
            },
          },
          header: ({ column }) => (
            <Button
              variant="ghost"
              className="group"
              onClick={() => {
                column.toggleSorting(
                  column.getIsSorted() ? column.getIsSorted() === 'asc' : true,
                );
              }}
            >
              Created At
              {column.getIsSorted() === 'asc' ? (
                <ChevronUp
                  className={cn(
                    'invisible ml-2 size-4 group-hover:visible',
                    column.getIsSorted() && 'visible',
                  )}
                />
              ) : (
                <ChevronDown
                  className={cn(
                    'invisible ml-2 size-4 group-hover:visible',
                    column.getIsSorted() && 'visible',
                  )}
                />
              )}
            </Button>
          ),
          cell: ({ getValue }) => {
            const value = getValue();
            return (
              <>
                <p className="mb-[3px] leading-none">
                  {format(value, 'MM-dd-yyyy')}
                </p>
                <p className="text-muted-foreground leading-none">
                  {format(value, 'h:mm:ss a')}
                </p>
              </>
            );
          },
        }),
        columnHelper.display({
          id: 'actions',
          meta: {
            style: {
              width: 'min',
              textAlign: 'right',
            },
          },
          cell: ({ row }) => (
            <>
              <Button asChild variant="ghost" className="size-8 p-0">
                <a
                  target="_blank"
                  href={`${import.meta.env.VITE_BACKEND_URL ?? ''}/api/storage/get/${row.original.id}/${row.original.name}?projectId=${client.projectId}`}
                  rel="noreferrer"
                >
                  <ExternalLink className="size-4" />
                </a>
              </Button>

              <Button
                variant="ghost"
                className="size-8 p-0"
                onClick={() => {
                  mutate(row.original.id);
                }}
              >
                <Trash2 className="size-4" />
              </Button>
            </>
          ),
        }),
      ] as ColumnDef<Upload>[],
    [client.projectId, loadedImages, mutate],
  );

  return (
    <div className="px-60 py-7 2xl:px-96">
      <Tabs className="w-full" defaultValue="files">
        <div className="flex w-full justify-between">
          <TabsList>
            <TabsTrigger value="files">Files</TabsTrigger>
            <TabsTrigger value="usage">Usage</TabsTrigger>
          </TabsList>

          <Dialog>
            <DialogTrigger asChild>
              <Button size="sm" className="w-20">
                Upload
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Upload files</DialogTitle>
              </DialogHeader>

              <div {...getRootProps()}>
                <input {...getInputProps()} />
                {isDragActive ? (
                  <p>Drop the files here ...</p>
                ) : (
                  <p>Drag 'n' drop some files here, or click to select files</p>
                )}
              </div>

              <DialogFooter>
                <Button
                  type="submit"
                  onClick={() => {
                    const formData = new FormData();
                    for (const file of acceptedFiles) {
                      formData.append('files[]', file);
                    }

                    void (async () => {
                      await client.json({
                        method: 'POST',
                        path: '/storage/upload',
                        projectIdNeeded: true,
                        body: formData,
                      });
                    })();
                  }}
                >
                  Upload
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>

        <TabsContent value="files">
          <DataTable data={storage} columns={columns} />
        </TabsContent>
        <TabsContent value="usage">Usage statistics.</TabsContent>
      </Tabs>
    </div>
  );
}

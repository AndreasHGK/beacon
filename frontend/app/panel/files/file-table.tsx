"use client"

import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { ClipboardCopy, Download, MoreHorizontal, Trash } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from "@/components/ui/dialog"
import prettyBytes from "pretty-bytes"
import { toast } from "sonner"
import { useCallback } from "react"
import { useRouter } from "next/navigation"

export type File = {
  file_id: string
  file_name: string
  file_size: number
  upload_date: Date
  url: string
  apiUrl: string
}

export const columns: ColumnDef<File>[] = [
  {
    accessorKey: "file_name",
    header: "Name",
  },
  {
    header: "Size",
    cell: ({ row }) => {
      return prettyBytes(row.original.file_size)
    },
  },
  {
    accessorKey: "upload_date",
    header: () => <div className="text-right">Upload Date</div>,
    cell: ({ row }) => {
      const date = row.original.upload_date.toLocaleString("en-GB", {
        year: "numeric",
        month: "long",
        day: "numeric",
        hour: "numeric",
        minute: "numeric",
        hour12: false,
      })
      return <div className="text-right">{date}</div>
    },
  },
  {
    id: "actions",
    cell: ({ row }) => {
      // eslint-disable-next-line react-hooks/rules-of-hooks
      const router = useRouter()

      const copyFileUrl = () => {
        const { origin } = new URL(window.location.href)
        navigator.clipboard.writeText(origin + row.original.url)
        toast("Copied file URL to clipboard.")
      }

      // eslint-disable-next-line react-hooks/rules-of-hooks
      const deleteFile = useCallback(async () => {
        const resp = await fetch(row.original.apiUrl, { method: "DELETE" })

        if (resp.status != 200) {
          toast("An error occurred trying to delete a file.")
          return
        }

        toast("The file has been deleted successfully.")
        router.refresh()
      }, [router, row.original.apiUrl])

      return (
        <Dialog>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="h-8 w-8 p-0">
                <span className="sr-only">Open menu</span>
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="end"
              onCloseAutoFocus={(e) => e.preventDefault()}
            >
              <DropdownMenuLabel>Actions</DropdownMenuLabel>
              <DropdownMenuItem>
                <a
                  className="flex flex-row"
                  href={row.original.apiUrl + "/content"}
                  download={row.original.file_name}
                >
                  <Download className="size-4" />
                  &nbsp; Download
                </a>
              </DropdownMenuItem>
              <DropdownMenuItem onSelect={copyFileUrl}>
                <ClipboardCopy className="size-4" />
                &nbsp; Copy URL
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DialogTrigger asChild>
                <DropdownMenuItem className="text-destructive">
                  <Trash className="size-4" />
                  &nbsp; Delete file
                </DropdownMenuItem>
              </DialogTrigger>
            </DropdownMenuContent>
          </DropdownMenu>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Are you sure?</DialogTitle>
              <DialogDescription>
                This action will permanently remove the file &quot
                {row.original.file_name}&quot. We will not be able to recover
                it.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button type="submit" onClick={deleteFile}>
                Delete file
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )
    },
  },
]

interface FileTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
}

export function FileTable<TValue>({
  columns,
  data,
}: FileTableProps<File, TValue>) {
  const table = useReactTable({
    data: data.map((item) => {
      return {
        ...item,
        url: `/files/${item.file_id}/${item.file_name}`,
        apiUrl: `/api/files/${item.file_id}/${item.file_name}`,
      }
    }),
    columns,
    getCoreRowModel: getCoreRowModel(),
  })

  return (
    <div className="rounded-md border w-full">
      <Table>
        <TableHeader>
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id}>
              {headerGroup.headers.map((header) => {
                return (
                  <TableHead key={header.id}>
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                  </TableHead>
                )
              })}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id}>
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
  )
}

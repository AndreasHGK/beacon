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
import { MoreHorizontal, Trash } from "lucide-react"
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
import { badgeVariants } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

export type User = {
  id: string
  username: string
  total_storage_space: number
  created_at: Date
  is_admin: boolean
}

export const columns: ColumnDef<User>[] = [
  {
    id: "username",
    accessorKey: "username",
    header: "Username",
    cell: ({ row }) => {
      if (row.original.is_admin) {
        return (
          <p>
            {row.original.username}{" "}
            <span className={cn(badgeVariants({ variant: "secondary" }))}>
              admin
            </span>
          </p>
        )
      } else {
        return row.original.username
      }
    },
  },
  {
    header: "Total Storage Used",
    cell: ({ row }) => {
      return prettyBytes(row.original.total_storage_space)
    },
  },
  {
    accessorKey: "Created At",
    header: () => <div className="text-right">Created At</div>,
    cell: ({ row }) => {
      const date = row.original.created_at.toLocaleString("en-GB", {
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

      // eslint-disable-next-line react-hooks/rules-of-hooks
      const deleteUser = useCallback(async () => {
        const resp = await fetch(`/api/users/${row.original.id}`, {
          method: "DELETE",
        })

        if (resp.status != 200) {
          toast("An error occurred trying to delete a user.")
          return
        }

        toast("The user has been deleted successfully.")
        router.refresh()
      }, [router, row.original.id])

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
              <DropdownMenuSeparator />
              <DialogTrigger asChild>
                <DropdownMenuItem className="text-destructive">
                  <Trash className="size-4" />
                  &nbsp; Delete user
                </DropdownMenuItem>
              </DialogTrigger>
            </DropdownMenuContent>
          </DropdownMenu>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Are you sure?</DialogTitle>
              <DialogDescription>
                This will <strong>permanently</strong> delete the user along
                with all their associated files.
              </DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button type="submit" onClick={deleteUser}>
                Delete user
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      )
    },
  },
]

interface UserTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
}

export function UserTable<TValue>({
  columns,
  data,
}: UserTableProps<User, TValue>) {
  const table = useReactTable({
    data,
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

import { redirect } from "next/navigation"
import { serverFetch } from "@/lib/server-fetch"
import { FileTable, columns } from "./file-table"
import { getSession } from "@/lib/sessions"

export default async function Panel() {
  let session = getSession()
  if (!session) {
    redirect("/login")
  }

  const resp = await serverFetch(`/api/users/${session?.uuid}/files`)

  if (resp.status == 401) {
    redirect("/login")
  }

  if (resp.status != 200) {
    throw new Error("Failed to fetch files")
  }

  const files = (await resp.json()).map(
    (item: { file_id: string; upload_date: number }) => {
      return {
        ...item,
        upload_date: new Date(item.upload_date),
      }
    }
  )

  return (
    <main className="flex flex-col justify-center flex-1 gap-8">
      <div className="flex flex-col gap-2">
        <h1 className="font-bold text-4xl">My Files</h1>
        <p className="text-lg text-muted-foreground">
          Manage your files. Uploading files is currently only supported using
          the command line interface.
        </p>
      </div>
      <FileTable columns={columns} data={files} />
    </main>
  )
}

import { redirect } from "next/navigation"
import { serverFetch } from "@/lib/server-fetch"
import { FileTable, columns } from "./file-table"
import { getSession } from "@/lib/sessions"
import { Dashboard } from "@/components/dashboard"

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
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>My Files</Dashboard.Title>
        <Dashboard.Subtext>
          Manage your files. Uploading files is currently only supported using
          the command line interface.
        </Dashboard.Subtext>
      </Dashboard.Header>
      <FileTable columns={columns} data={files} />
    </Dashboard.Page>
  )
}

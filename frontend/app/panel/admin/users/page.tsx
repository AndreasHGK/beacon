import { Dashboard } from "@/components/dashboard"
import { User, UserTable, columns } from "./user-table"
import { serverFetch } from "@/lib/server-fetch"
import { redirect } from "next/navigation"

export default async function Users() {
  const resp = await serverFetch("/api/users")

  if (resp.status == 401) {
    redirect("/login")
  }

  if (!resp.ok) {
    throw new Error("Failed to fetch files")
  }

  const files = (await resp.json()).map((item: any) => {
    return {
      ...item,
      created_at: new Date(item.created_at),
    }
  }) as User[]

  return (
    <Dashboard.Page>
      <Dashboard.Header>
        <Dashboard.Title>Users</Dashboard.Title>
        <Dashboard.Subtext>
          Manage users in this beacon instance.
        </Dashboard.Subtext>
      </Dashboard.Header>
      <UserTable columns={columns} data={files} />
    </Dashboard.Page>
  )
}

import { NavGroup, NavItem } from "@/components/nav"
import { serverFetch } from "@/lib/server-fetch"
import { mustGetSession } from "@/lib/sessions"
import { redirect } from "next/navigation"

export default async function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const session = mustGetSession()

  let isAdmin = false
  const resp = await serverFetch(`/api/users/${session.uuid}/admin`)
  if (resp.status == 401) {
    redirect("/login")
  }

  if (!resp.ok) {
    throw new Error("could not get admin status")
  }

  if ((await resp.text()) === "true") {
    isAdmin = true
  }

  return (
    <div className="max-w-screen-xl w-full flex flex-row mx-auto pt-16 px-4">
      <div className="flex-none">
        <div className="rounded-md bg-card flex flex-col gap-4 grow shrink-0">
          <div className="w-48" />
          <NavGroup title="Dashboard">
            <NavItem href="/panel" content="Overview" />
            <NavItem href="/panel/files" content="My Files" />
          </NavGroup>
          <NavGroup title="Account">
            <NavItem href="/panel/account/profile" content="Profile" />
            <NavItem href="/panel/account/security" content="Security" />
          </NavGroup>
          {isAdmin ? (
            <NavGroup title="Admin">
              <NavItem href="/panel/admin/users" content="Users" />
            </NavGroup>
          ) : (
            <></>
          )}
        </div>
      </div>
      <div className="grow flex-1 px-6">{children}</div>
    </div>
  )
}

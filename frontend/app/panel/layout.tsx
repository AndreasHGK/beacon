"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { ReactNode } from "react"

function NavItem(props: { href: string; content: string }) {
  const pathname = usePathname()

  return (
    <Link
      href={props.href}
      className={
        "text-muted-foreground pl-1 " +
        (pathname == props.href
          ? "font-medium text-foreground"
          : "text-muted-foreground")
      }
    >
      {props.content}
    </Link>
  )
}

function NavGroup(props: { title?: string; children: ReactNode }) {
  return (
    <div className="gap-1 flex flex-col">
      <h3 className="text-foreground font-bold">{props.title || ""}</h3>
      {props.children}
    </div>
  )
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
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
        </div>
      </div>
      <div className="grow flex-1 px-6">{children}</div>
    </div>
  )
}

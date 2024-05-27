"use client"

import Link from "next/link"
import { usePathname } from "next/navigation"
import { ReactNode } from "react"

export function NavItem(props: { href: string; content: string }) {
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

export function NavGroup(props: { title?: string; children: ReactNode }) {
  return (
    <div className="gap-1 flex flex-col">
      <h3 className="text-foreground font-bold">{props.title || ""}</h3>
      {props.children}
    </div>
  )
}

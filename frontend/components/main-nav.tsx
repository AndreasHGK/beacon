import { type VariantProps } from "class-variance-authority"
import Link from "next/link"
import { ModeToggle } from "@/components/mode-toggle"
import { hasSession } from "@/lib/sessions"
import { buttonVariants } from "@/components/ui/button"
import { getConfig } from "@/lib/config"

export async function UserNav(props: VariantProps<typeof buttonVariants>) {
  const config = await getConfig()
  if (hasSession()) {
    return (
      <Link
        href="/panel"
        className={buttonVariants({ variant: props.variant })}
      >
        Dashboard
      </Link>
    )
  }

  return (
    <div className="flex flex-row gap-2">
      <Link
        href="/login"
        className={buttonVariants({ variant: props.variant })}
      >
        Log in
      </Link>
      {config.allow_registering ? (
        <Link
          href="/register"
          className={buttonVariants({ variant: props.variant })}
        >
          Register
        </Link>
      ) : (
        <></>
      )}
    </div>
  )
}

export function MainNav() {
  return (
    <header className="sticky top-0 border-b border-border/50 z-10 py-2">
      <div className="container flex flex-row items-center bg-background">
        <Link href="/" className="text-2xl font-semibold pl-8 pr-8">
          Beacon
        </Link>
        <div className="flex flex-row w-full pr-8 justify-end self-stretch items-center gap-2">
          <UserNav variant="outline" />
          <ModeToggle />
        </div>
      </div>
    </header>
  )
}

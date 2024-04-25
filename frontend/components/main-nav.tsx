import Link from "next/link"
import { ModeToggle } from "@/components/mode-toggle"
import { hasSession } from "@/lib/sessions"
import { buttonVariants } from "@/components/ui/button"

export function MainNav() {
  return (
    <header className="sticky top-0 border-b border-border/50 z-10 py-2">
      <div className="container flex flex-row items-center bg-background">
        <Link href="/" className="text-2xl font-semibold pl-8 pr-8">
          Beacon
        </Link>
        <div className="flex flex-row w-full pr-8 justify-end self-stretch items-center gap-2">
          {(() => {
            if (hasSession()) {
              return (
                <Link
                  href="/panel"
                  className={buttonVariants({ variant: "outline" })}
                >
                  Dashboard
                </Link>
              )
            } else {
              return (
                <div className="flex flex-row gap-2">
                  <Link
                    href="/login"
                    className={buttonVariants({ variant: "outline" })}
                  >
                    Log in
                  </Link>
                  <Link
                    href="/register"
                    className={buttonVariants({ variant: "outline" })}
                  >
                    Register
                  </Link>
                </div>
              )
            }
          })()}
          <ModeToggle />
        </div>
      </div>
    </header>
  )
}

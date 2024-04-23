import Link from "next/link"
import { Button } from "@/components/ui/button"
import { ModeToggle } from "@/components/mode-toggle"

export function MainNav() {
  return (
    <header className="sticky top-0 border-b border-border/50 z-10 py-2">
      <div className="container flex flex-row items-center bg-background">
        <Link href="/" className="text-2xl font-semibold pl-8 pr-8">
          Beacon
        </Link>
        <div className="flex flex-row w-full pr-8 justify-end self-stretch items-center gap-1">
          <div className="flex flex-row gap-1">
            <Button variant="outline">
              <p>Register</p>
            </Button>
            <Button variant="outline">
              <p>Log in</p>
            </Button>
          </div>
          <ModeToggle />
        </div>
      </div>
    </header>
  )
}

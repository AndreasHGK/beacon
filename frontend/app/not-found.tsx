import { Separator } from "@/components/ui/separator"

export default function NotFound() {
  return (
    <main className="flex flex-col justify-center flex-1">
      <div className="flex items-center justify-center flex-col">
        <div className="flex flex-col gap-1 text-center">
          <h1 className="text-5xl font-extrabold tracking-tight">404</h1>
          <Separator />
          <h2 className="text-2xl tracking-wide">Page not found</h2>
        </div>
      </div>
    </main>
  )
}

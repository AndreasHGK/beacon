import { LoaderCircle } from "lucide-react"

export default function Loading() {
  return (
    <main className="flex flex-col justify-center flex-1">
      <div className="flex items-center justify-center flex-col gap-4">
        <div className="flex flex-row gap-2">
          <LoaderCircle className="animate-spin" />
          <p>Loading...</p>
        </div>
      </div>
    </main>
  )
}

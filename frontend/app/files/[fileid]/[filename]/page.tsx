import { FilePreview } from "@/components/file-preview"
import { buttonVariants } from "@/components/ui/button"
import { Download } from "lucide-react"
import Link from "next/link"
import { notFound } from "next/navigation"

export default async function File({
  params,
}: {
  params: { fileid: string; filename: string }
}) {
  const res = await fetch(
    `http://localhost:4000/api/files/${params.fileid}/${params.filename}`
  )
  if (res.status == 404 || res.status == 400) {
    return notFound()
  }

  if (!res.ok) {
    throw new Error("Failed to fetch data")
  }

  const fileInfo = await res.json()

  return (
    <main className="flex flex-col justify-center flex-1">
      <div className="flex items-center justify-center flex-col gap-4">
        <FilePreview name={params.filename} size={fileInfo["file_size"]} />
        <Link
          href={`/api/files/${params.fileid}/${params.filename}/content`}
          className={buttonVariants({ variant: "default" })}
        >
          <div className="flex flex-row gap-2 text-lg">
            <Download />
            Download
          </div>
        </Link>
      </div>
    </main>
  )
}

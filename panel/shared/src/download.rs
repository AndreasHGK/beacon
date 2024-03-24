use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;

use crate::file::FileId;

#[derive(Params, PartialEq, Eq)]
pub struct FileParams {
    file_id: Option<FileId>,
    name: Option<String>,
}

#[component]
pub fn Download() -> impl IntoView {
    let params = use_params::<FileParams>();

    let id = move || {
        params.with(|p| {
            p.as_ref()
                .map(|v| v.file_id)
                .unwrap_or_default()
                .unwrap_or_default()
        })
    };
    let id_str = move || id().to_string();
    let name = move || {
        params.with(|p| {
            p.as_ref()
                .map(|v| v.name.clone())
                .unwrap_or_default()
                .unwrap_or_default()
        })
    };

    // Determine what icon to use from the file extension.
    let icon = move || match name().split('.').last().unwrap_or_default() {
        "txt" | "toml" | "yaml" | "yml" | "json" => icondata::BsFileEarmarkTextFill,
        "tar" | "gz" | "zip" | "rar" | "7z" => icondata::BsFileEarmarkZipFill,
        "png" | "jpg" | "jpeg" | "webm" => icondata::BsFileEarmarkImageFill,
        "mp3" | "wav" | "flac" | "aac" => icondata::BsFileEarmarkMusicFill,
        "xlsx" | "xls" | "csv" => icondata::BsFileEarmarkSpreadsheetFill,
        "rtf" | "typ" | "tex" => icondata::BsFileEarmarkRichtextFill,
        "mp4" | "mov" | "avi" => icondata::BsFileEarmarkPlayFill,
        "docx" | "doc" => icondata::BsFileEarmarkWordFill,
        "otf" | "ttf" => icondata::BsFileEarmarkFontFill,
        "pptx" | "ppt" => icondata::BsFileEarmarkPptFill,
        "pdf" => icondata::BsFileEarmarkPdfFill,
        _ => icondata::BsFileEarmarkFill,
    };

    view! {
        <div class="flex justify-center items-center h-screen bg-gray-800">
            <div class="flex flex-col justify-center items-center pb-6">
                <div class="pb-4">
                    {move || view! {
                        <Icon
                            icon={icon()}
                            width="14em"
                            height="14em"
                            class="text-gray-200 mx-auto"
                        />
                    }}
                </div>
                <p class="text-3xl font-semibold text-gray-100 pb-1">{name}</p>
                <p class="text-xl text-gray-400 pb-6">"2.5 MB"</p>
                <a
                    class="
                        bg-blue-400
                        px-6
                        py-2
                        rounded-2xl
                        text-4xl
                        text-gray-100
                        font-bold
                        select-none
                    "
                    href={move || format!("/files/{}/{}/content", id_str(), name())}
                    target="_parent"
                    download
                >
                    "Download"
                </a>
            </div>
        </div>
    }
}

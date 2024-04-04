use leptos::*;
use leptos_icons::Icon;
use leptos_router::*;

use crate::file::{FileId, FileInfo};

#[server(GetFileInfo)]
async fn get_file_info(
    file_id: FileId,
    file_name: String,
) -> Result<Option<FileInfo>, ServerFnError> {
    use crate::server::{file::FileDb, state::get_state};
    use leptos::server_fn::error::NoCustomError;
    use std::sync::Arc;

    let file_db: Arc<FileDb> = get_state()?;
    let Some(info) = file_db.file_info(file_id).await.map_err(|err| {
        log::error!("Error reading file db: {err}");
        ServerFnError::<NoCustomError>::ServerError("internal error".into())
    })?
    else {
        return Ok(None);
    };

    if info.file_name != file_name {
        return Ok(None);
    }

    Ok(Some(info))
}

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
    let name = move || {
        params.with(|p| {
            p.as_ref()
                .map(|v| v.name.clone())
                .unwrap_or_default()
                .unwrap_or_default()
        })
    };

    let file_info = create_resource(|| (), move |_| get_file_info(id(), name()));

    let inner_html = move || {
        file_info.get().map(|file_info| match file_info {
            Err(err) => {
                log::error!("error getting file info: {err}");
                view! {
                    <Icon
                        icon={icondata::BiErrorCircleSolid}
                        width="14em"
                        height="14em"
                        class="text-gray-200 mx-auto"
                    />
                    <p class="text-3xl font-semibold text-gray-100 pb-3">
                        "We're sorry! Something went wrong..."
                    </p>
                }
            }
            Ok(None) => {
                view! {
                    <Icon
                        icon={icondata::TbError404}
                        width="14em"
                        height="14em"
                        class="text-gray-200 mx-auto"
                    />
                    <p class="text-3xl font-semibold text-gray-100 pb-3">
                        "We could not find that file."
                    </p>
                }
            }
            Ok(Some(file_info)) => {
                // Determine what icon to use from the file extension.
                let icon = match file_info.file_name.split('.').last().unwrap_or_default() {
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

                let file_size = format!("{} Bytes", file_info.file_size);

                view! {
                    <div class="pb-4">
                        <Icon
                            icon={icon}
                            width="14em"
                            height="14em"
                            class="text-gray-200 mx-auto"
                        />
                    </div>
                    <p class="text-3xl font-semibold text-gray-100 pb-1">
                        {file_info.file_name.clone()}
                    </p>
                    <p class="text-xl text-gray-400 pb-6">{file_size}</p>
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
                        href={format!(
                            "/files/{}/{}/content",
                            file_info.file_id,
                            file_info.file_name,
                        )}
                        target="_parent"
                        download
                    >
                        "Download"
                    </a>
                }
            }
        })
    };

    let loading = move || {
        view! {
            <Icon
                icon={icondata::BsFileEarmarkFill}
                width="14em"
                height="14em"
                class="text-gray-200 mx-auto animate-pulse"
            />
        }
    };

    view! {
        <div class="flex justify-center items-center h-screen">
            <div class="flex flex-col justify-center items-center pb-6">
                <Suspense fallback=loading>
                    {inner_html}
                </Suspense>
            </div>
        </div>
    }
}

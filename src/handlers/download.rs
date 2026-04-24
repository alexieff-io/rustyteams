use cef::*;

wrap_download_handler! {
    pub struct TeamsDownloadHandler;

    impl DownloadHandler {
        fn on_before_download(
            &self,
            _browser: Option<&mut Browser>,
            _download_item: Option<&mut DownloadItem>,
            suggested_name: Option<&CefString>,
            callback: Option<&mut BeforeDownloadCallback>,
        ) -> i32 {
            let Some(callback) = callback else { return 0; };
            let suggested = suggested_name.map(CefString::to_string).unwrap_or_default();

            // Prompt with the native Save dialog. Passing an empty path with
            // show_dialog=1 tells CEF to surface its own system dialog; we go
            // one better and use rfd so we control the starting directory.
            let picked = rfd::FileDialog::new()
                .set_file_name(&suggested)
                .set_directory(dirs::download_dir().unwrap_or_else(|| std::path::PathBuf::from(".")))
                .save_file();

            match picked {
                Some(path) => {
                    let p = CefString::from(path.to_string_lossy().as_ref());
                    callback.cont(Some(&p), 0);
                }
                None => {
                    // User cancelled. Passing empty path with show_dialog=0 aborts.
                    let empty = CefString::from("");
                    callback.cont(Some(&empty), 0);
                }
            }
            1
        }

        fn on_download_updated(
            &self,
            _browser: Option<&mut Browser>,
            _download_item: Option<&mut DownloadItem>,
            _callback: Option<&mut DownloadItemCallback>,
        ) {
            // Default behavior is fine — CEF tracks progress internally and
            // writes to the path we supplied above.
        }
    }
}

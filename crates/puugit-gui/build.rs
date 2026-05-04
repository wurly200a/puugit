fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/puugit-icon.ico");
        res.compile().unwrap();
    }
}

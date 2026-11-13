// Gives the Windows executable its icon.
fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../packaging/y2e.ico");
        res.set("ProductName", "YouTube to Everything");
        res.compile().expect("could not embed the Windows icon");
    }
}

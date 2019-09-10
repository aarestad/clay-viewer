use std::{fs, cmp::max, ffi::OsStr};
use image;
use clay_core;


fn from_os(os_str: &OsStr) -> String {
    os_str.to_string_lossy().into_owned()
}

pub fn save_screenshot(data: &[u8], size: (usize, usize), ll: bool) -> clay_core::Result<String> {
    fs::create_dir_all("screenshots")?;
    let maxn = fs::read_dir("screenshots")?
    .filter_map(|f_| f_.ok().map(|f| f.path()).and_then(|p| {
        p.file_stem().map(|s| from_os(s))
        .and_then(|n| n.parse::<i32>().ok())
    }))
    .fold(0, |b, n| max(b, n)) + 1;

    let ext = if ll { "png" } else { "jpg" };

    let filename = format!("screenshots/{:04}.{}", maxn, ext);
    image::save_buffer(&filename, &data, size.0 as u32, size.1 as u32, image::RGB(8))?;
    Ok(filename)
}

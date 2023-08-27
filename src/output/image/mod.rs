use std::ffi::{c_int, c_char, CString};



extern "C"{
    #[link_name = "stbi_write_png"]
    fn _stbi_write_png(filename: *const c_char, w: c_int, h: c_int, comp: c_int, data: *const u8, stride_in_bytes: c_int) -> c_int;

    #[link_name = "stbi_write_jpg"]
    fn _stbi_write_jpg(filename: *const c_char, w: c_int, h: c_int, comp: c_int, data: *const u8, quality: c_int) -> c_int;
}

/// Creates a png image and returns a non-zero number on success.
/// 
/// filename: The file to write to.
/// 
/// w: Width of the image.
/// 
/// h: Height of the image.
/// 
/// comp: Components per pixel:
/// 1=Y, 2=YA, 3=RGB, 4=RGBA (Y is monochrome color.)
/// 
/// data: The raw pixels data in bytes.
/// 
/// stride: The number of bytes between a row of pixels and the first byte of the next row.
pub fn stbi_write_png(filename: &str, w:u32, h:u32, comp:u32, data: &[u8], stride: u32) -> i32{
    unsafe{
        let f = CString::new(filename).expect("kr");
        _stbi_write_png(f.as_ptr(), w as i32, h as i32, comp as i32, data.as_ptr(), stride as i32)
    }
}


/// Creates a jpeg image and returns a non-zero number on success.
/// 
/// filename: The file to write to.
/// 
/// w: Width of the image.
/// 
/// h: Height of the image.
/// 
/// comp: Components per pixel:
/// 1=Y, 2=YA, 3=RGB, 4=RGBA (Y is monochrome color.)
/// 
/// JPEG does ignore alpha channels in input data.
/// 
/// data: The raw pixels data in bytes.
/// 
/// quality is between 1 and 100. Higher quality looks better but results in a bigger image.
pub fn stbi_write_jpg(filename: &str, w:u32, h:u32, comp:u32, data: &[u8], quality: u32) -> i32{
    unsafe{
        let f = CString::new(filename).expect("kr");
        _stbi_write_jpg(f.as_ptr(), w as i32, h as i32, comp as i32, data.as_ptr(), quality as i32)
    }
}

fn main(){
    cc::Build::new()
        .file("./src/output/image/lib.c")
        .compile("stb_image_write");
}
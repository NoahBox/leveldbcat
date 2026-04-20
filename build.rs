use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=resource/LevelDBCat_LOGO.png");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let logo_path = manifest_dir.join("resource").join("LevelDBCat_LOGO.png");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let icon_path = out_dir.join("leveldbcat.ico");
    let rc_path = out_dir.join("leveldbcat.rc");

    let resized = image::open(&logo_path)?.thumbnail(256, 256).to_rgba8();
    let mut canvas = image::RgbaImage::from_pixel(256, 256, image::Rgba([0, 0, 0, 0]));
    let x = (canvas.width() - resized.width()) / 2;
    let y = (canvas.height() - resized.height()) / 2;
    image::imageops::overlay(&mut canvas, &resized, i64::from(x), i64::from(y));
    image::DynamicImage::ImageRgba8(canvas)
        .save_with_format(&icon_path, image::ImageFormat::Ico)?;

    let icon_path = icon_path.to_string_lossy().replace('\\', "\\\\");
    fs::write(&rc_path, format!("1 ICON \"{icon_path}\"\n"))?;

    let _ = embed_resource::compile(&rc_path, embed_resource::NONE);
    Ok(())
}

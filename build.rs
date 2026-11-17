use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let assets = manifest.join("assets");
    println!("cargo:rerun-if-changed=assets");

    match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("windows") => embed_windows_resources(&assets, &out),
        Ok("linux") => write_tray_pixmaps(&assets.join("shut-it-up.png"), &out),
        _ => {}
    }
}

fn embed_windows_resources(assets: &Path, out: &Path) {
    let icon = assets.join("shut-up.ico").display().to_string().replace('\\', "\\\\");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let numeric = version.replace('.', ",");
    let description = env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default();

    let script = format!(
        r#"1 ICON "{icon}"

1 VERSIONINFO
FILEVERSION {numeric},0
PRODUCTVERSION {numeric},0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "FileDescription", "Shut up"
      VALUE "Comments", "{description}"
      VALUE "FileVersion", "{version}"
      VALUE "ProductName", "Shut up"
      VALUE "ProductVersion", "{version}"
      VALUE "OriginalFilename", "shut-up.exe"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0409, 1200
  END
END
"#
    );

    let rc = out.join("shut-up.rc");
    fs::write(&rc, script).unwrap();
    embed_resource::compile(&rc, embed_resource::NONE)
        .manifest_optional()
        .unwrap();
}

fn write_tray_pixmaps(png: &Path, out: &Path) {
    let decoder = png::Decoder::new(BufReader::new(File::open(png).unwrap()));
    let mut reader = decoder.read_info().unwrap();
    let mut rgba = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut rgba).unwrap();
    assert_eq!(info.color_type, png::ColorType::Rgba, "the logo has to be RGBA");

    for size in [22, 32, 48] {
        let argb = downscale(&rgba, info.width as usize, size);
        fs::write(out.join(format!("tray-{size}.argb")), argb).unwrap();
    }
}

fn downscale(rgba: &[u8], source: usize, target: usize) -> Vec<u8> {
    let mut argb = Vec::with_capacity(target * target * 4);
    for ty in 0..target {
        for tx in 0..target {
            let (x0, x1) = (tx * source / target, (tx + 1) * source / target);
            let (y0, y1) = (ty * source / target, (ty + 1) * source / target);
            let mut sum = [0u64; 4];
            for y in y0..y1 {
                for x in x0..x1 {
                    let p = &rgba[(y * source + x) * 4..][..4];
                    let alpha = p[3] as u64;
                    sum[0] += p[0] as u64 * alpha;
                    sum[1] += p[1] as u64 * alpha;
                    sum[2] += p[2] as u64 * alpha;
                    sum[3] += alpha;
                }
            }
            let count = ((x1 - x0) * (y1 - y0)) as u64;
            let alpha = sum[3] / count;
            let channel = |c: u64| c.checked_div(sum[3]).unwrap_or(0) as u8;
            argb.extend([alpha as u8, channel(sum[0]), channel(sum[1]), channel(sum[2])]);
        }
    }
    argb
}

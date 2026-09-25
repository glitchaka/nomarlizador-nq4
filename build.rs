mod icon_design;

use std::{
    env,
    fs::File,
    path::PathBuf,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icon_design.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR no definido"));
    let icon_path = out_dir.join("normalizador-nq4.ico");

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in [16_u32, 24, 32, 48, 64, 128, 256] {
        let image = ico::IconImage::from_rgba_data(size, size, icon_design::rgba_icon(size));
        let entry = ico::IconDirEntry::encode(&image).expect("No se pudo codificar el icono");
        icon_dir.add_entry(entry);
    }

    let file = File::create(&icon_path).expect("No se pudo crear el recurso de icono");
    icon_dir.write(file).expect("No se pudo escribir el archivo ICO");

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon(
        icon_path
            .to_str()
            .expect("La ruta del icono contiene caracteres inválidos"),
    );
    resource
        .compile()
        .expect("No se pudo incrustar el icono en el ejecutable de Windows");
}

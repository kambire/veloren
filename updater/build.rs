// Incrusta el icono de World of Azeria y los datos del programa en el .exe,
// igual que hace el cliente del juego (voxygen/build.rs).
#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    // Con el identificador 1 el lanzador lo carga también como icono de la ventana
    res.set_icon_with_id("../assets/voxygen/logo.ico", "1");
    res.set("ProductName", "World of Azeria");
    res.set("FileDescription", "World of Azeria - Lanzador");
    res.set("CompanyName", "World of Azeria");
    res.set("LegalCopyright", "Basado en Veloren (GPL-3.0)");
    res.compile().expect("No se pudo incrustar el icono del lanzador");
}

#[cfg(not(windows))]
fn main() {}

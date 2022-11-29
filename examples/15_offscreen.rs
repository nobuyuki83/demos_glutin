use del_gl::gl as gl;

fn main() {
    let rnd = del_gl::glutin::off_screen_render::OffScreenRender::new(300, 300);
    let mut drawer = del_gl::array_vtxcolor::Drawer { program: 0, mode: gl::TRIANGLES };

    drawer.compile_shader(&rnd.gl);
    let vtx2xyzrgb: Vec<f32> = vec![
        -0.5, -0.5, 1.0, 0.0, 0.0,
        0.0, 0.5, 0.0, 1.0, 0.0,
        0.5, -0.5, 0.0, 0.0, 1.0];
    drawer.initialize(
        &rnd.gl,
        &vtx2xyzrgb);
    rnd.start();
    drawer.draw_frame(&rnd.gl);
    let data: Vec<u8> = rnd.save();
    image::save_buffer(
        &std::path::Path::new("target/image.png"),
        &data, rnd.width, rnd.height, image::ColorType::Rgb8).unwrap();
}
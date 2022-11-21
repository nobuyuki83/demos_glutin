use del_gl::gl as gl;
use image::io::Reader as ImageReader;
use image::EncodableLayout;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::mesh_tex::Drawer::new();
    {
        let tri2vtx_xyz: Vec<usize>;
        let vtx2xyz: Vec<f32>;
        let tri2vtx_uv: Vec<usize>;
        let vtx2uv: Vec<f32>;
        {
            let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
            let filename: &str = "asset/Babi/babi.obj";
            obj.load(filename);
            let num_elem = obj.elem2idx.len() - 1;
            println!("element size: {}", num_elem);
            assert_eq!(obj.elem2idx[num_elem - 1], (num_elem - 1) * 3);  // triangle mesh
            tri2vtx_xyz = obj.idx2vtx_xyz;
            vtx2xyz = del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(obj.vtx2xyz, 3);
            vtx2uv = obj.vtx2uv;
            tri2vtx_uv = obj.idx2vtx_uv;
        };
        println!("vertex size: {}", vtx2xyz.len() / 3);
        println!("uv size: {}", vtx2uv.len() / 2);
        {
            let img = ImageReader::open("asset/Babi/tex.png").unwrap();
            println!("{:?}", img.format());
            let img = img.decode().unwrap();
            let img = image::imageops::flip_vertical(&img);
            println!("{:?}", img.dimensions());
            // println!("{:?}",img.color());
            unsafe {
                del_gl::utility::gen_texture(&viewer.gl,
                                             img.width().try_into().unwrap(),
                                             img.height().try_into().unwrap(),
                                             img.as_bytes(), gl::RGBA);
            }
        }
        //
        drawer.compile_shader(&viewer.gl);
        {
            let (uni2xyz, uni2uv, tri2uni, _, _)
                =  del_msh::unify_index::unify_separate_trimesh_indexing_xyz_uv(
                &vtx2xyz, &vtx2uv,
                &tri2vtx_xyz,
                &tri2vtx_uv);
            drawer.update_vertex(&viewer.gl, &uni2xyz, 3);
            drawer.set_texture_uv(&viewer.gl, &uni2uv);
            drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2uni, None);
        }
    }

    // this clousure captures drawer, viewer and 'move' them. drawer and viewer cannot be usable anymore
    let event_handle_closure = move |event: glutin::event::Event<()>,
                                     _elwt: &glutin::event_loop::EventLoopWindowTarget<()>,
                                     control_flow: &mut glutin::event_loop::ControlFlow| {
        viewer.event_handle(&event);
        use glutin::event_loop::ControlFlow::{Wait, ExitWithCode};
        if viewer.should_close { *control_flow = ExitWithCode(0); } else { *control_flow = Wait; }
        //
        if viewer.should_draw {
            let mat_projection = viewer.nav.projection_matrix(
                viewer.ui_state.win_width, viewer.ui_state.win_height);
            let mat_modelview = viewer.nav.modelview_matrix();
            drawer.draw(
                &viewer.gl,
                mat_modelview.as_slice(), // nalgebra is column major same as OpenGL
                mat_projection.as_slice()); // nalgebra is column major same as OpenGL
            viewer.windowed_context.swap_buffers().unwrap();
        }
    };
    event_loop.run(event_handle_closure);
}

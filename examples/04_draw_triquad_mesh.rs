use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::mesh::Drawer::new();
    {
        let elem2idx: Vec<usize>;
        let idx2vtx_xyz: Vec<usize>;
        let vtx2xyz: Vec<f32> = {
            let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
            let filename: &str = "asset/HorseSwap.obj";
            obj.load(filename);
            elem2idx = obj.elem2idx;
            idx2vtx_xyz = obj.idx2vtx_xyz;
            del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(obj.vtx2xyz, 3)
        };
        println!("vertex size: {}", vtx2xyz.len() / 3);
        println!("element size: {}", elem2idx.len() -1 );
        unsafe {
            drawer.compile_shader(&viewer.gl);
            drawer.update_vertex(&viewer.gl, &vtx2xyz, 3);
        }
        {
            let (tri2vtx,_) = del_msh::tri2vtx::from_tri_quad_mesh(
                &elem2idx, &idx2vtx_xyz);
            unsafe {
                drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx, [1., 0., 0.]);
            }
        }
        {
            let line2vtx: Vec<usize> = del_msh::line2vtx::from_tri_quad_mesh(
                &elem2idx, &idx2vtx_xyz,
                vtx2xyz.len() / 3);
            unsafe {
                drawer.add_element(&viewer.gl, gl::LINES, &line2vtx, [0., 0., 0.]);
            }
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

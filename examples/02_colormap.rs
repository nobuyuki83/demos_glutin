use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::drawer_meshposcolor::DrawerMeshPosColor::new();
    {
        let filename: &str = "asset/bunny_1k.obj";
        let tri2vtx: Vec<usize>;
        let vtx2xyz: Vec<f32>;
        {
            let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
            obj.load(filename);
            println!("vertex size: {}", obj.vtx2xyz.len() / 3);
            println!("element size: {}", obj.elem2vtx_idx.len() - 1);
            tri2vtx = obj.elem2vtx_xyz;
            vtx2xyz = obj.vtx2xyz.iter().map(|v| *v * 0.03 ).collect();
        }
        let mut vtx2val = Vec::<f32>::new();
        vtx2val.resize(vtx2xyz.len() / 3, 0.);
        for iv in 0..vtx2xyz.len() / 3 {
            vtx2val[iv] = ((vtx2xyz[iv * 3 + 0] * 5.).sin()+1.)*0.5;
        }
        drawer.color_map = vec![
            [0.0, 0.0, 0.0], // 0
            [0.5, 0.0, 0.0], // 0.2
            [1.0, 0.0, 0.0], // 0.4
            [1.0, 0.5, 0.0], // 0.6
            [1.0, 1.0, 0.0], // 0.8
            [1.0, 1.0, 1.0]]; // 1.0
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx2xyz, 3);
        drawer.update_value(&viewer.gl, &vtx2val);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri2vtx);
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

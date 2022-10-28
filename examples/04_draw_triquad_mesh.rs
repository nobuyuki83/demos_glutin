use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer3::Viewer3::open();

    let mut drawer = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let elem_vtx_xyz_index: Vec<usize>;
        let elem_vtx_xyz: Vec<usize>;
        let vtx_xyz: Vec<f32> = {
            let mut obj = del_msh::io_obj::WavefrontObj::<f32>::new();
            let filename: &str = "asset/HorseSwap.obj";
            obj.load(filename);
            elem_vtx_xyz = obj.elem_vtx_xyz.iter().map(|i| *i as usize).collect();
            elem_vtx_xyz_index = obj.elem_vtx_index.iter().map(|i| *i as usize).collect();
            del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(obj.vtx_xyz, 3)
        };
        println!("vertex size: {}", vtx_xyz.len() / 3);
        println!("element size: {}", elem_vtx_xyz_index.len() -1 );

        let tri_vtx = del_msh::topology_mix::meshtri_from_meshtriquad(
            &elem_vtx_xyz_index, &elem_vtx_xyz);

        let line_vtx: Vec<usize> = del_msh::topology_mix::meshline_from_meshtriquad(
            &elem_vtx_xyz_index, &elem_vtx_xyz,
            vtx_xyz.len() / 3);

        use crate::gl::types::GLuint;
        let tri_vtx0: Vec<GLuint> = tri_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
        let line_vtx0: Vec<GLuint> = line_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx_xyz, 3);
        drawer.add_element(&viewer.gl, gl::TRIANGLES, &tri_vtx0, [1., 0., 0.]);
        drawer.add_element(&viewer.gl, gl::LINES, &line_vtx0, [0., 0., 0.]);
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

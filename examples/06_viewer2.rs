use del_gl::gl as gl;

fn main() {
    let (mut viewer, event_loop) = del_gl::glutin::viewer2::Viewer2::open();

    let mut drawer = del_gl::drawer_meshpos::DrawerMeshPos::new();
    {
        let (mut vtx_xyz, quad_vtx) = del_msh::primitive::grid_quad2::<f32>(
            32,32);
        vtx_xyz = del_misc::nalgebra::msh_misc::centerize_normalize_boundingbox(vtx_xyz, 2);
        use crate::gl::types::GLuint;
        drawer.compile_shader(&viewer.gl);
        drawer.update_vertex(&viewer.gl, &vtx_xyz, 2);
        {
            let tri_vtx = del_msh::topology_uniform::tri_from_quad(&quad_vtx);
            let elem_vtx0: Vec<GLuint> = tri_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
            drawer.add_element(&viewer.gl, gl::TRIANGLES, &elem_vtx0, [1., 0., 0.]);
        }
        {
            let line_vtx: Vec<usize> = del_msh::topology_uniform::mshline(
                &quad_vtx, 4,
                &[0, 1, 1, 2, 2, 3, 3, 0],
                vtx_xyz.len() / 2);
            let line_vtx0: Vec<GLuint> = line_vtx.iter().map(|i| *i as gl::types::GLuint).collect();
            drawer.add_element(&viewer.gl, gl::LINES, &line_vtx0, [0., 0., 0.]);
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
            let mat_projection = viewer.nav.projection_matrix(viewer.ui_state.win_width, viewer.ui_state.win_height);
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
